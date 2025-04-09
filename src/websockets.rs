use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::thread;
use std::time::Duration;
use urlencoding::encode;

#[derive(Serialize)]
struct Authenticate<'a> {
    name: &'a str,
    args: Vec<Args<'a>>,
}

#[derive(Serialize)]
struct Args<'a> {
    headers: Headers<'a>,
}

#[derive(Serialize)]
struct Headers<'a> {
    #[serde(rename = "X-Ms-Test-User")]
    x_ms_test_user: &'a str,
    #[serde(rename = "Authorization")]
    authorization: &'a str,
    #[serde(rename = "X-MS-Migration")]
    x_ms_migration: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub name: String,
    pub args: Option<Vec<Arg>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arg {
    pub dropped_indicators: Option<Vec<DroppedIndicator>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DroppedIndicator {
    pub tag: String,
    pub etag: String,
}

use reqwest::{header::HeaderMap, Client};
use url::form_urlencoded;

fn teams_trouter_register_one(
    skype_token: &str,
    endpoint: &str,
    app_id: &str,
    template_key: &str,
    path: &str,
) -> Result<(), reqwest::Error> {
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

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let _res = client
        .post("https://edge.skype.com/registrar/prod/v2/registrations")
        .header("Content-Type", "application/json")
        .header("X-Skypetoken", skype_token)
        .body(body.to_string())
        .send()?;

    Ok(())
}

struct trouter_connection_info {
    socketio: String,
    surl: String,
    ccid: Option<String>,
    connectparams: Value,
}

fn teams_trouter_start(
    endpoint: &str,
    skype_token: &str,
) -> Result<trouter_connection_info, reqwest::Error> {
    let url = format!(
        "https://go.trouter.teams.microsoft.com/v4/a?epid={}",
        endpoint
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Length", "0".parse().unwrap());
    headers.insert("X-Skypetoken", skype_token.parse().unwrap());

    let res = client.post(&url).headers(headers).body("").send()?;
    let text = res.text()?;

    let value: Value = serde_json::from_str(&text).expect("Invalid JSON");

    Ok(trouter_connection_info {
        socketio: value.get("socketio").unwrap().as_str().unwrap().to_string(),
        surl: value.get("surl").unwrap().as_str().unwrap().to_string(),
        ccid: value
            .get("ccid")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        connectparams: value.get("connectparams").unwrap().to_owned(),
    })
}

fn teams_trouter_get_sessionid(url: &str, skype_token: &str) -> Result<String, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Skypetoken", skype_token.parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send()?;
    let text = res.text()?;
    let session_id = text.split(":").nth(0).unwrap();

    Ok(session_id.to_string())
}

fn websockets() {
    let endpoint = "94e0c9c2-8408-4d38-995b-1cf5f4e14fef";

    let skype_token = "";

    let connection_info = teams_trouter_start(endpoint, skype_token).unwrap();

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

    let session_id = teams_trouter_get_sessionid(&url, skype_token).unwrap();

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
    .unwrap();

    let ssw_path = format!("{}/{}", surl_trimmed, "SkypeSpacesWeb");
    teams_trouter_register_one(
        skype_token,
        endpoint,
        "SkypeSpacesWeb",
        "SkypeSpacesWeb_2.3",
        &ssw_path,
    )
    .unwrap();

    teams_trouter_register_one(
        skype_token,
        endpoint,
        "TeamsCDLWebWorker",
        "TeamsCDLWebWorker_1.9",
        &connection_info.surl,
    )
    .unwrap();

    //let handle = thread::spawn(move || {
    //start_ws(&websocket_url.replace("https://", "wss://"));
    //});

    //handle.join().expect("WebSocket thread panicked");
}

use iced::futures;
use iced::stream;
use iced::widget::text;

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};

use async_tungstenite::tungstenite;
use std::fmt;

pub fn connect() -> impl Stream<Item = String> {
    stream::channel(100, |mut output| async move {
        let mut state = State::Disconnected;
        println!("Once?");
        loop {
            match &mut state {
                State::Disconnected => {
                    const ECHO_SERVER: &str = "ws://127.0.0.1:3030";

                    match async_tungstenite::tokio::connect_async(ECHO_SERVER).await {
                        Ok((websocket, _)) => {
                            let (sender, receiver) = mpsc::channel(100);

                            let _ = output.send("Hello5".to_string()).await;

                            state = State::Connected(websocket, receiver);
                        }
                        Err(_) => {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            let _ = output.send("Hello4".to_string()).await;
                        }
                    }
                }
                State::Connected(websocket, input) => {
                    let mut fused_websocket = websocket.by_ref().fuse();

                    futures::select! {
                        received = fused_websocket.select_next_some() => {
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                   let _ = output.send("Hello1".to_string()).await;
                                }
                                Err(_) => {
                                    let _ = output.send("Hello2".to_string()).await;

                                    state = State::Disconnected;
                                }
                                Ok(_) => continue,
                            }
                        }

                        message = input.select_next_some() => {
                            let result = websocket.send(tungstenite::Message::Text(message.to_string().into())).await;

                            if result.is_err() {
                                let _ = output.send("Hello3".to_string()).await;

                                state = State::Disconnected;
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum State {
    Disconnected,
    Connected(
        async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>,
        mpsc::Receiver<Message>,
    ),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

impl Connection {
    pub fn send(&mut self, message: Message) {
        self.0
            .try_send(message)
            .expect("Send message to echo server");
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected,
    Disconnected,
    User(String),
}

impl Message {
    pub fn new(message: &str) -> Option<Self> {
        if message.is_empty() {
            None
        } else {
            Some(Self::User(message.to_string()))
        }
    }

    pub fn connected() -> Self {
        Message::Connected
    }

    pub fn disconnected() -> Self {
        Message::Disconnected
    }

    pub fn as_str(&self) -> &str {
        match self {
            Message::Connected => "Connected successfully!",
            Message::Disconnected => "Connection lost... Retrying...",
            Message::User(message) => message.as_str(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> text::IntoFragment<'a> for &'a Message {
    fn into_fragment(self) -> text::Fragment<'a> {
        text::Fragment::Borrowed(self.as_str())
    }
}
