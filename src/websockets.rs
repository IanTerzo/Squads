use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::thread;
use std::time::Duration;
use tungstenite::{connect, Message};
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

fn start_ws(url: &str) {
    let (mut socket, response) = connect(url).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());

    // https://ic3.teams.office.com/Teams.AccessAsUser.All https://ic3.teams.office.com/.default
    let token = "";
    let token = format!("Bearer {}", token);

    let message = Authenticate {
        name: "user.authenticate",
        args: vec![Args {
            headers: Headers {
                x_ms_test_user: "False",
                authorization: &token,
                x_ms_migration: "True",
            },
        }],
    };

    let json_str = serde_json::to_string(&message).unwrap();
    let final_message = format!("5:::{}", json_str);

    socket.send(Message::Text(final_message.into())).unwrap();

    let ping = r#"5:4+::{"name":"ping"}"#;

    let mut initial_message_batch = vec![];
    loop {
        let msg = socket.read().expect("Error reading message");

        let msg_text = msg.into_text().unwrap().to_string();
        println!("Received: {}", msg_text);

        let message_id = msg_text
            .find(|c| c == '[' || c == '{')
            .map(|idx| &msg_text[..idx])
            .unwrap_or(&msg_text);

        let body = msg_text.strip_prefix(message_id).unwrap();

        let chain_id = message_id.split(":").nth(0).unwrap();
        let chain_order = message_id.split(":").nth(1).unwrap();
        // Auth chain
        if chain_id == "5" {
            let mut message_r: Reply = serde_json::from_str(body).expect("Error deserializing");
            if message_r.name == "trouter.connected" {
                let response = format!("5:{}+::{{\"name\":\"user.activity\",\"args\":[{{\"state\":\"active\",\"cv\":\"2zAuo1xx7w6IaNkQ5VxMHQ.0.1\"}}]}}", chain_order);
                initial_message_batch.push(response);
            } else if message_r.name == "trouter.message_loss" {
                message_r.name = "trouter.processed_message_loss".to_string();

                let message_loss_reply = format!(
                    "5:{}+::{}",
                    chain_order,
                    serde_json::to_string(&message_r).expect("Error serializing"),
                );

                initial_message_batch.push(message_loss_reply.clone());
            }
        }

        if chain_id == "5" && chain_order == "3" {
            thread::sleep(Duration::from_millis(400));
            for message in initial_message_batch.clone() {
                socket.send(Message::Text(message.clone().into())).unwrap();
                println!("Sent: {}", message)
            }

            thread::sleep(Duration::from_millis(300));

            let ping = r#"5:4+::{"name":"ping"}"#;
            socket.send(Message::Text(ping.into()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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

        let handle = thread::spawn(move || {
            start_ws(&websocket_url.replace("https://", "wss://"));
        });

        handle.join().expect("WebSocket thread panicked");
    }
}
