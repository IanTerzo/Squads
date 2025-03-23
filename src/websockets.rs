use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use tungstenite::{connect, Message};

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

use reqwest::Client;
use url::form_urlencoded;

// Generate endpoint (random?) ->
// Subscribe (/v2/users/ME/endpoints/%s) ->
// Begin trouter (https://go.trouter.teams.microsoft.com/v4/a?) and get socketio ->
// Fetch socketio for sessionid ->
// connect to socketio websocket with sessionid and con_num (arbitrary: 1234567890123_u64) and (endpoint)

// Endpoint is random?
// sa->endpoint = purple_uuid_random();
async fn send_trouter_request(
    endpoint: &str,
    skype_token: &str,
    client: &Client,
) -> Result<(), reqwest::Error> {
    // Build the URL with the encoded endpoint
    let encoded_epid: String = form_urlencoded::byte_serialize(endpoint.as_bytes()).collect();
    let url = format!(
        "https://go.trouter.teams.microsoft.com/v4/a?epid={}",
        encoded_epid
    );

    // Send POST request with headers
    let response = client
        .post(&url)
        .header("x-skypetoken", skype_token)
        .header("Content-Length", "0")
        .body("") // Empty body to match Content-Length: 0
        .send()
        .await?;

    // Optionally handle response status or body
    println!("Status: {}", response.status());

    Ok(())
}

fn start_ws() {
    let url = "wss://pub-ent-dewc-04-t.trouter.teams.microsoft.com/v4/c?tc=%7B%22cv%22:%222025.07.01.5%22,%22ua%22:%22TeamsCDL%22,%22hr%22:%22%22,%22v%22:%221415/25030201008%22%7D&timeout=40&epid=5ec9e271-a754-4ec7-b780-c2ae6dcac9b1&ccid=&cor_id=35eabb8f-4d78-4f19-928b-92f926721c10&con_num=1742740397159_0";
    let (mut socket, response) = connect(url).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());

    let token = "-";
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
        start_ws();
    }
}
