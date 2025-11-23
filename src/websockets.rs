use async_tungstenite::tokio::ConnectStream;
use async_tungstenite::{tungstenite, WebSocketStream};
use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};
use iced::futures;
use iced::futures::stream::SplitStream;
use iced::stream;
use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use urlencoding::encode;

use crate::api::{self, AccessToken};
use crate::auth::{get_or_gen_skype_token, get_or_gen_token};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum State {
    Disconnected,
    Connected(SplitStream<WebSocketStream<ConnectStream>>),
}

// Can be expanded in case headers are needed
#[derive(Debug, Deserialize)]
struct WebsocketResponseWrapper {
    body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketResponse {
    Connected(ConnectionInfo),
    Message(WebsocketMessage),
    Presences(Presences),
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub endpoint: String,
    pub surl: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presences {
    pub presence: Vec<Presence>,
    pub is_snapshot: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presence {
    pub mri: String,
    pub etag: String,
    pub source: Option<String>,
    pub presence: PresenceInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceInfo {
    pub source_network: Option<String>,
    pub calendar_data: Option<CalendarData>,
    pub availability: Option<String>,
    pub activity: Option<String>,
    pub device_type: Option<String>,
    pub last_active_time: Option<String>,
    pub note: Option<Note>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarData {
    pub is_out_of_office: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub message: String,
    pub publish_time: String,
    pub expiry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketMessage {
    pub time: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub resource_link: String,
    pub resource_type: String,
    pub resource: api::Message,
    pub isactive: bool,
}

struct TrouterConnectionInfo {
    socketio: String,
    surl: String,
    ccid: Option<String>,
    connectparams: Value,
}

pub async fn websockets_subscription(
    token: &AccessToken,
    endpoint: &str,
    surl: &str,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://teams.microsoft.com/ups/emea/v1/pubsub/subscriptions/{}",
        endpoint
    );

    let mut headers = HeaderMap::new();

    let access_token = format!("Bearer {}", token.value);

    headers.insert("authorization", access_token.parse().unwrap());

    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("x-ms-client-user-agent", "Teams-V2-Web".parse().unwrap());
    headers.insert("x-ms-client-version", "1415/25040319109".parse().unwrap());
    headers.insert("x-ms-correlation-id", "1".parse().unwrap());
    headers.insert("x-ms-endpoint-id", endpoint.parse().unwrap());

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.post(url).headers(headers).body(body).send().await?;

    if res.status().is_success() {
        Ok(())
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

async fn teams_trouter_start(
    endpoint: &str,
    skype_token: &str,
) -> Result<TrouterConnectionInfo, Box<dyn std::error::Error>> {
    let url = format!(
        "https://go.trouter.teams.microsoft.com/v4/a?epid={}",
        endpoint
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Length", "0".parse()?);
    headers.insert("X-Skypetoken", skype_token.parse()?);

    let res = client.post(&url).headers(headers).body("").send().await?;

    if res.status().is_success() {
        let text = res.text().await?;

        let value: Value = serde_json::from_str(&text).expect("Invalid JSON");

        Ok(TrouterConnectionInfo {
            socketio: value.get("socketio").unwrap().as_str().unwrap().to_string(),
            surl: value.get("surl").unwrap().as_str().unwrap().to_string(),
            ccid: value
                .get("ccid")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            connectparams: value.get("connectparams").unwrap().to_owned(),
        })
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

async fn teams_trouter_get_sessionid(
    url: &str,
    skype_token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Skypetoken", skype_token.parse()?);

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let text = res.text().await?;
        let session_id = text.split(":").nth(0).unwrap();

        Ok(session_id.to_string())
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

async fn teams_trouter_register_one(
    skype_token: &str,
    endpoint: &str,
    app_id: &str,
    template_key: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = json!({
        "clientDescription": {
            "appId": app_id,
            "aesKey": "",
            "languageId": "en-US",
            "platform": "edge",
            "templateKey": template_key,
            "platformUIVersion": "49/24062722442",

        },
        "registrationId": endpoint,
        "nodeId": "",
        "transports": {
            "TROUTER": [{
                "context": "",
                "path": path,
                "ttl": 86400,
            }]
        },
    });

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client
        .post("https://edge.skype.com/registrar/prod/v2/registrations")
        .header("Content-Type", "application/json")
        .header("X-Skypetoken", skype_token)
        .body(body.to_string())
        .send()
        .await?;

    if !res.status().is_success() {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        return Err(error_message.into());
    }

    Ok(())
}

async fn begin_websockets(skype_token: &str, endpoint: &str) -> (String, String) {
    let connection_info = teams_trouter_start(endpoint, skype_token).await.unwrap();

    let mut url = format!("{}socket.io/1/?v=v4&", connection_info.socketio);

    if let Some(params) = connection_info.connectparams.as_object() {
        for (key, value) in params {
            if let Some(val_str) = value.as_str() {
                url.push_str(&format!("{}={}&", key, encode(val_str)));
            }
        }
    }

    let tc_value =
        r#"{"cv":"TEAMS_TROUTER_TCCV","ua":"TeamsCDL","hr":"","v":"TEAMS_CLIENTINFO_VERSION"}"#;
    url.push_str(&format!("tc={}&", encode(tc_value)));
    url.push_str(&format!("con_num={}_{}&", 1234567890123_i64, 1));
    url.push_str(&format!("epid={}&", encode(endpoint)));

    if let Some(ccid) = &connection_info.ccid {
        url.push_str(&format!("ccid={}&", encode(ccid)));
    }

    url.push_str("auth=true&timeout=40&");

    let session_id = teams_trouter_get_sessionid(&url, skype_token)
        .await
        .unwrap();

    let mut websocket_url = format!(
        "{}socket.io/1/websocket/{}?v=v4&",
        connection_info.socketio, session_id
    );

    if let Some(params) = connection_info.connectparams.as_object() {
        for (key, value) in params {
            if let Some(val_str) = value.as_str() {
                websocket_url.push_str(&format!("{}={}&", key, encode(val_str)));
            }
        }
    }

    let tc_value =
        r#"{"cv":"TEAMS_TROUTER_TCCV","ua":"TeamsCDL","hr":"","v":"TEAMS_CLIENTINFO_VERSION"}"#;
    websocket_url.push_str(&format!("tc={}&", encode(tc_value)));
    websocket_url.push_str(&format!("con_num={}_{}&", 1234567890123_i64, 1));
    websocket_url.push_str(&format!("epid={}&", encode(endpoint)));

    if let Some(ccid) = &connection_info.ccid {
        url.push_str(&format!("ccid={}&", encode(ccid)));
    }

    websocket_url.push_str("auth=true&timeout=40&");

    let surl_trimmed = connection_info.surl.trim_end_matches('/');

    let ngc_path = format!("{}/{}", surl_trimmed, "NGCallManagerWin");
    teams_trouter_register_one(
        skype_token,
        endpoint,
        "NextGenCalling",
        "DesktopNgc_2.3:SkypeNgc",
        &ngc_path,
    )
    .await
    .unwrap();

    let ssw_path = format!("{}/{}", surl_trimmed, "SkypeSpacesWeb");
    teams_trouter_register_one(
        skype_token,
        endpoint,
        "SkypeSpacesWeb",
        "SkypeSpacesWeb_2.3",
        &ssw_path,
    )
    .await
    .unwrap();

    teams_trouter_register_one(
        skype_token,
        endpoint,
        "TeamsCDLWebWorker",
        "TeamsCDLWebWorker_1.9",
        &connection_info.surl,
    )
    .await
    .unwrap();

    (
        websocket_url.replace("https://", "wss://"),
        connection_info.surl,
    )
}

pub fn connect(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    tenant: String,
) -> impl Stream<Item = WebsocketResponse> {
    stream::channel(100, |mut output| async move {
        let mut state = State::Disconnected;
        loop {
            match &mut state {
                State::Disconnected => {
                    let access_token = get_or_gen_token(
                        access_tokens.clone(),
                        "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                        &tenant,
                    )
                    .await;
                    let skype_token =
                        get_or_gen_skype_token(access_tokens.clone(), access_token).await;

                    let endpoint = "3feae13d-a16c-48f1-b52a-d417ebd07a29";

                    let (url, surl) = begin_websockets(&skype_token.value, endpoint).await;
                    match async_tungstenite::tokio::connect_async(url).await {
                        Ok((websocket, _)) => {
                            let (mut write, read) = websocket.split();

                            let _ = output
                                .send(WebsocketResponse::Connected(ConnectionInfo {
                                    endpoint: endpoint.to_string(),
                                    surl,
                                }))
                                .await;

                            // Ping every 40 seconds to keep the connection alive
                            tokio::spawn(async move {
                                loop {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
                                    if let Err(e) =
                                        write.send(tungstenite::Message::Text("ping".into())).await
                                    {
                                        eprintln!("Failed to send ping: {}", e);
                                        break;
                                    }
                                }
                            });

                            state = State::Connected(read);
                        }
                        Err(e) => {
                            eprintln!("Failed to connect: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                }
                State::Connected(read) => {
                    let mut fused_websocket = read.by_ref().fuse();

                    futures::select! {
                        received = fused_websocket.select_next_some() => {
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                    if let Some(json_content) = message.as_str().find('{').map(|i| &message[i..]) {
                                        if let Ok(wrapper) = serde_json::from_str::<WebsocketResponseWrapper>(json_content) {
                                            let json_body = wrapper.body;

                                            if let Ok(message_t) = serde_json::from_str(&json_body) {
                                                let _ = output.send(WebsocketResponse::Message(message_t)).await;
                                            }
                                            else if let Ok(presences) = serde_json::from_str(&json_body) {
                                                let _ = output.send(WebsocketResponse::Presences(presences)).await;
                                            }
                                            else {
                                                let _ = output.send(WebsocketResponse::Other(message.to_string())).await;
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Websocket connection failed.");
                                    state = State::Disconnected;
                                }
                                Ok(_) => continue,
                            }
                        }
                    }
                }
            }
        }
    })
}
