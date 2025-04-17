pub use crate::api_types::*; // expose the type
use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const LOG_REQUESTS: bool = false;
const TEAMS_CLIENT_ID: &str = "1fec8e78-bce4-4aaf-ab1b-5451cc387264";

fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub async fn gen_device_code(
    tenant_id: String,
) -> Result<DeviceCodeInfo, Box<dyn std::error::Error>> {
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/devicecode",
        tenant_id
    );
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    let body = format!(
        "client_id={}&\
        resource=https://api.spaces.skype.com",
        TEAMS_CLIENT_ID
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.post(url).headers(headers).body(body).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<DeviceCodeInfo, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

pub async fn gen_refresh_token_from_device_code(
    device_code: String,
    tenant_id: String,
) -> Result<AccessToken, Box<dyn std::error::Error>> {
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/token",
        tenant_id
    );

    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id={}&\
    code={}&\
    grant_type=urn:ietf:params:oauth:grant-type:device_code",
        TEAMS_CLIENT_ID, device_code
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.post(url).headers(headers).body(body).send().await?;

    if !res.status().is_success() {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        return Err(error_message.into());
    }

    let token_data: HashMap<String, Value> = res.json().await?;
    if let (Some(value), Some(expires_in)) = (
        token_data.get("refresh_token"),
        token_data.get("expires_in"),
    ) {
        let refresh_token = AccessToken {
            value: value.as_str().unwrap().to_string(),
            expires: get_epoch_s() + expires_in.as_str().unwrap().parse::<u64>().unwrap(),
        };

        Ok(refresh_token)
    } else {
        Err("Response does not have a value or a refresh_token.".into())
    }
}

pub async fn gen_refresh_token_from_code(
    code: String,
    code_verifier: String,
    tenant_id: String,
) -> Result<AccessToken, Box<dyn std::error::Error>> {
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id={}&\
    redirect_uri=https://teams.microsoft.com/v2&\
    scope=https://ic3.teams.office.com/.default openid profile offline_access&\
    code={}&\
    code_verifier={}&\
    grant_type=authorization_code&\
    claims={{\"access_token\":{{\"xms_cc\":{{\"values\":[\"CP1\"]}}}}}}",
        TEAMS_CLIENT_ID, code, code_verifier
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.post(url).headers(headers).body(body).send().await?;

    if !res.status().is_success() {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        return Err(error_message.into());
    }

    let token_data: HashMap<String, Value> = res.json().await?;
    if let (Some(value), Some(expires_in)) = (
        token_data.get("refresh_token"),
        token_data.get("expires_in"),
    ) {
        let refresh_token = AccessToken {
            value: value.as_str().unwrap().to_string(),
            expires: get_epoch_s() + expires_in.as_u64().unwrap(),
        };

        Ok(refresh_token)
    } else {
        Err("Response does not have a value or a refresh_token.".into())
    }
}

pub async fn renew_refresh_token(
    refresh_token: &AccessToken,
    tenant_id: String,
) -> Result<AccessToken, String> {
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id={}&\
        scope=openid profile offline_access&\
        grant_type=refresh_token&\
        client_info=1&\
        x-client-SKU=msal.js.browser&\
        x-client-VER=3.7.1&\
        refresh_token={}",
        TEAMS_CLIENT_ID, refresh_token.value
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let res = client
        .post(url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .unwrap();

    if !res.status().is_success() {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await.unwrap()
        );
        return Err(error_message);
    }

    let token_data: HashMap<String, Value> = res.json().await.unwrap();

    if let (Some(value), Some(expires_in)) = (
        token_data.get("refresh_token"),
        token_data.get("expires_in"),
    ) {
        let refresh_token = AccessToken {
            value: value.as_str().unwrap().to_string(),
            expires: get_epoch_s() + expires_in.as_u64().unwrap(),
        };

        Ok(refresh_token)
    } else {
        Err("Response does not have a expiration or a refresh_token.".to_string())
    }
}

pub async fn gen_token(
    refresh_token: &AccessToken,
    scope: String,
    tenant_id: String,
) -> Result<AccessToken, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id={}&\
        scope={} openid profile offline_access&\
        grant_type=refresh_token&\
        client_info=1&\
        x-client-SKU=msal.js.browser&\
        x-client-VER=3.7.1&\
        refresh_token={}&\
        claims={{\"access_token\":{{\"xms_cc\":{{\"values\":[\"CP1\"]}}}}}}",
        TEAMS_CLIENT_ID, scope, refresh_token.value
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .post(format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant_id
        ))
        .headers(headers)
        .body(body)
        .send()
        .await
        .unwrap();

    if !res.status().is_success() {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await.unwrap()
        );
        return Err(error_message);
    }

    let token_data: HashMap<String, Value> = res.json().await.unwrap();
    if let (Some(value), Some(expires_in)) =
        (token_data.get("access_token"), token_data.get("expires_in"))
    {
        let access_token = AccessToken {
            value: value.as_str().unwrap().to_string(),
            expires: get_epoch_s() + expires_in.as_u64().unwrap(),
        };

        Ok(access_token)
    } else {
        Err("Response does not have a expiration date or a access_token.".to_string())
    }
}
// Api: Emea v2
// Scope: https://api.spaces.skype.com/Authorization.ReadWrite
async fn user_aggregate_settings(
    token: &AccessToken,
    json_body: HashMap<String, bool>,
) -> Result<HashMap<String, Value>, anyhow::Error> {
    let url = "https://teams.microsoft.com/api/mt/part/emea-02/beta/users/useraggregatesettings";
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .headers(headers)
        .json(&json_body)
        .send()
        .await
        .context("Failed to send request to user aggregate settings API")?;

    if res.status().is_success() {
        let json_response: HashMap<String, Value> = res
            .json()
            .await
            .context("Failed to deserialize JSON response")?;
        Ok(json_response)
    } else {
        let status = res.status();
        let body = res
            .text()
            .await
            .context("Failed to read error response body")?;
        Err(anyhow::anyhow!(
            "Request failed with status code {}: {}",
            status,
            body
        ))
    }
}

//async fn gen_web_url(token: &AccessToken) -> Result<(), anyhow::Error> {
//  let mut query = HashMap::new();
// query.insert("tenantSiteUrl".to_string(), true);

//let aggregate_settings = user_aggregate_settings(token, query).await.unwrap();

//let unformatted_web_url = aggregate_settings
//        .get("tenantSiteUrl")
//      .and_then(|tenant_site_url| tenant_site_url.get("value"))
//    .and_then(|value| value.get("webUrl"))
//  .ok_or_else(|| anyhow!("webUrl key is missing"))?;

//let web_url = unformatted_web_url
//  .to_string()
//.replace('\"', "") // Removing any double quotes from the string
//.replace("/_layouts/15/sharepoint.aspx", ""); // Adjusting the URL to the correct format

// let store = Store::new(get_config_path());
// store.set_data("web_url", Value::from(web_url));

//Ok(())
//}

// Api: Sharepoint
// Scope: format!("{}/.default", web_url)
async fn gen_spoidcrl(
    token: &AccessToken,
    section: String,
    web_url: Value,
) -> Result<AccessToken, anyhow::Error> {
    let url = format!(
        "{}/sites/{}/_api/SP.OAuth.NativeClient/Authenticate",
        web_url.to_string().replace('\"', ""),
        section
    );

    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );
    headers.insert("Content-Length", "0".parse().unwrap());

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .headers(headers)
        .send()
        .await
        .context("Failed to send request to user aggregate settings API")?;

    if res.status().is_success() {
        let headers = res.headers();

        if let Some(cookie) = headers.get("set-cookie") {
            let parsed_cookie = cookie
                .to_str()?
                .to_string()
                .replace("SPOIDCRL=", "")
                .replace("; path=/; secure; HttpOnly", "");

            let access_token = AccessToken {
                value: parsed_cookie,
                expires: get_epoch_s() + 2628288, // Doesn't change
            };
            Ok(access_token)
        } else {
            Err(anyhow!("Couldn't get response cookies"))
        }
    } else {
        let status = res.status();
        let body = res
            .text()
            .await
            .context("Failed to read error response body")?;
        Err(anyhow::anyhow!(
            "Request failed with status code {}: {}",
            status,
            body
        ))
    }
}

// Api: Authsvc v1
// Scope: https://api.spaces.skype.com/Authorization.ReadWrite
pub async fn gen_skype_token(
    token: &AccessToken,
) -> Result<AccessToken, Box<dyn std::error::Error>> {
    let url = "https://teams.microsoft.com/api/authsvc/v1.0/authz";
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let req_access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&req_access_token)?,
    );
    headers.insert("Content-Length", "0".parse()?);

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.post(url).headers(headers).send().await?;

    if !res.status().is_success() {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        return Err(error_message.into());
    }

    let token_data: HashMap<String, Value> = res.json().await?;

    if let Some(tokens) = token_data.get("tokens") {
        if let (Some(value), Some(expires_in)) = (tokens.get("skypeToken"), tokens.get("expiresIn"))
        {
            let refresh_token = AccessToken {
                value: value.as_str().unwrap().to_string(),
                expires: get_epoch_s() + expires_in.as_u64().unwrap(),
            };

            Ok(refresh_token)
        } else {
            Err("Response does not have a token or a expiration.".into())
        }
    } else {
        Err("Response include tokens".into())
    }
}

// Api: Emea v2
// Scope: https://chatsvcagg.teams.microsoft.com/.default
pub async fn teams_me(token: &AccessToken) -> Result<UserDetails, Box<dyn std::error::Error>> {
    let url = "https://teams.microsoft.com/api/csa/emea/api/v2/teams/users/me";
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let params = [
        ("isPrefetch", "false"),
        ("enableMembershipSummary", "true"),
        ("enableRC2Fetch", "false"),
    ];

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<UserDetails, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v1
// Scope: TODO
pub async fn properties(token: &AccessToken) -> Result<UserProperties, Box<dyn std::error::Error>> {
    let url = "https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/properties";
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<UserProperties, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v1
// Scope: https://ic3.teams.office.com/.default
pub async fn conversations(
    token: &AccessToken,
    thread_id: String,
    message_id: Option<u64>,
) -> Result<Conversations, Box<dyn std::error::Error>> {
    let thread_part = if let Some(msg_id) = message_id {
        format!("{};messageid={}", thread_id, msg_id)
    } else {
        thread_id.clone()
    };

    let url = format!(
        "https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/conversations/{}/messages?pageSize=200",
        thread_part
    );

    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body)?;
        let pretty_json = serde_json::to_string_pretty(&parsed_body)?;
        let result: Result<Conversations, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                println!("Error occurred while serializing: {}", err);
                if let Some(line) = pretty_json.lines().nth(err.line().saturating_sub(1)) {
                    println!("Line: {}", line);
                }
                Err(Box::new(err))
            }
        }
    } else {
        let status = res.status();
        let body = res.text().await?;
        let error_message = format!("Status code: {}, Response body: {}", status, body);
        Err(error_message.into())
    }
}

// Api: Emea v2
// Scope: https://chatsvcagg.teams.microsoft.com/.default
pub async fn fetch_short_profile(
    token: &AccessToken,
    user_ids: Vec<String>,
) -> Result<FetchShortProfile, Box<dyn std::error::Error>> {
    let url = "https://teams.microsoft.com/api/mt/part/emea-02/beta/users/fetchShortProfile";
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_str("application/json;charset=UTF-8").unwrap(),
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let params = [
        ("isMailAddress", "false"),
        ("enableGuest", "true"),
        ("skypeTeamsInfo", "true"),
        ("canBeSmtpAddress", "false"),
        ("includeIBBarredUsers", "true"),
        ("includeDisabledAccounts", "true"),
        ("useSkypeNameIfMissing", "false"),
    ];

    let body = format!("[\"{}\"]", user_ids.join("\",\""));

    let res = client
        .post(url)
        .headers(headers)
        .query(&params)
        .body(body)
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<FetchShortProfile, serde_json::Error> =
            serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}
// Api: Graph
// Scope: https://graph.microsoft.com/.default
pub async fn me(token: &AccessToken) -> Result<Profile, Box<dyn std::error::Error>> {
    let url = "https://graph.microsoft.com/v1.0/me";
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_str("application/json;charset=UTF-8")?,
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<Profile, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Graph
// Scope: https://graph.microsoft.com/.default
pub async fn users(token: &AccessToken) -> Result<Users, Box<dyn std::error::Error>> {
    let url = "https://graph.microsoft.com/v1.0/users?$top=999";
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_str("application/json;charset=UTF-8")?,
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<Users, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v2
// Scope: https://chatsvcagg.teams.microsoft.com/.default
pub async fn team_conversations(
    token: &AccessToken,
    team_id: String,
    topic_id: String,
) -> Result<TeamConversations, Box<dyn std::error::Error>> {
    let url = format!(
        "https://teams.microsoft.com/api/csa/emea/api/v2/teams/{}/channels/{}",
        team_id, topic_id
    );
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<TeamConversations, serde_json::Error> =
            serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                eprintln!("Line: {}", line_content);

                Err(err.into())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Graph
// Scope: https://graph.microsoft.com/.default
async fn team_channel_info(
    token: &AccessToken,
    group_id: String,
    topic_id: String,
) -> Result<HashMap<String, Value>, String> {
    let url = format!(
        "https://graph.microsoft.com/beta/teams/{}/channels/{}",
        group_id, topic_id
    );
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let client = reqwest::Client::new();
    let res = client.get(url).headers(headers).send().await.unwrap();

    if res.status().is_success() {
        let parsed = res.json::<HashMap<String, Value>>().await.unwrap();
        Ok(parsed)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await.unwrap()
        );
        Err(error_message)
    }
}

// Api: Emea v2
// Scope: https://api.spaces.skype.com/.default
async fn document_libraries(
    token: &AccessToken,
    skype_token: AccessToken,
    topic_id: String,
) -> Result<Vec<Value>, String> {
    let url = format!(
        "https://teams.microsoft.com/api/mt/part/emea-02/beta/channels/{}/documentlibraries",
        topic_id
    );
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );
    headers.insert("x-skypetoken", skype_token.value.parse().unwrap());

    let client = reqwest::Client::new();
    let res = client.get(url).headers(headers).send().await.unwrap();

    if res.status().is_success() {
        let parsed = res.json::<Vec<Value>>().await.unwrap();
        Ok(parsed)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await.unwrap()
        );
        Err(error_message)
    }
}

// Api: Sharepoint
// Scope: SPOIDCRL
async fn render_list_data_as_stream(
    token: &AccessToken,
    web_url: String,
    section: String,
    files_relative_path: String,
) -> Result<HashMap<String, Value>, String> {
    let url = format!(
        "{}/sites/{}/_api/web/GetListUsingPath(DecodedUrl=@a1)/RenderListDataAsStream?@a1='{}'&RootFolder={}&TryNewExperienceSingle=TRUE",
        web_url.to_string().replace('\"', ""),
        section,
        files_relative_path,
        files_relative_path
    );
    if LOG_REQUESTS {
        println!("Log: POST {}", url);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("cookie"),
        HeaderValue::from_str(format!("SPOIDCRL={}", token.value).as_str()).unwrap(),
    );
    headers.insert("Content-Length", "0".parse().unwrap());

    let mut params = HashMap::new();
    params.insert("RenderOptions".to_string(), Value::from(5723911));
    params.insert(
        "AllowMultipleValueFilterForTaxonomyFields".to_string(),
        Value::from(true),
    );
    params.insert("AddRequiredFields".to_string(), Value::from(true));
    params.insert("ModernListBoot".to_string(), Value::from(true));
    params.insert("RequireFolderColoringFields".to_string(), Value::from(true));

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .headers(headers)
        .query(&params)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let parsed = res.json::<HashMap<String, Value>>().await.unwrap();
        Ok(parsed)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await.unwrap()
        );
        Err(error_message)
    }
}

// Api: Emea v2
// Scope: https://chatsvcagg.teams.microsoft.com/.default
pub async fn authorize_team_picture(
    token: &AccessToken,
    group_id: String,
    etag: String,
    display_name: String,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let url = format!(
            "https://teams.microsoft.com/api/mt/part/emea-02/beta/users/15de4241-e9be-4910-a60f-3f37dd8652b8/profilepicturev2/teams/{}",
            group_id
	);
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("referer"),
        HeaderValue::from_static("https://teams.microsoft.com/v2/"),
    );
    headers.insert(
        HeaderName::from_static("cookie"),
        HeaderValue::from_str(
            format!(
                "authtoken=Bearer={}&Origin=https://teams.microsoft.com;",
                token.value
            )
            .as_str(),
        )?,
    );

    let params = [("etag", etag), ("displayName", display_name)];

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v2
// Scope: https://api.spaces.skype.com/Authorization.ReadWrite
pub async fn authorize_profile_picture(
    token: &AccessToken,
    user_id: String,
    display_name: String,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let url = format!(
        "https://teams.microsoft.com/api/mt/part/emea-02/beta/users/{}/profilepicturev2",
        user_id
    );
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("referer"),
        HeaderValue::from_static("https://teams.microsoft.com/_"),
    );
    headers.insert(
        HeaderName::from_static("cookie"),
        HeaderValue::from_str(
            format!(
                "authtoken=Bearer={}&Origin=https://teams.microsoft.com;",
                token.value
            )
            .as_str(),
        )?,
    );

    let params = [
        ("displayname", display_name),
        ("size", "HR64x64".to_string()),
    ];

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v2
// Scope: Skype
// Supports: https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/, https://eu-api.asm.skype.com/v1/objects/
pub async fn authorize_image(
    token: &AccessToken,
    url: String,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let access_token = format!("skype_token {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v2
// Scope: Skype
pub async fn authorize_merged_profile_picture(
    token: &AccessToken,
    users: Vec<(String, String)>,
    user_id: String,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let url = format!("https://teams.microsoft.com/api/mt/part/emea-02/beta/users/{user_id}/mergedProfilePicturev2");
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("cookie"),
        format!(
            "authtoken=Bearer={}&origin=https://teams.microsoft.com;",
            token.value
        )
        .parse()?,
    );

    headers.insert("Referer", "https://teams.microsoft.com/v2/".parse()?);

    let json_array: Vec<Value> = users
        .iter()
        .map(|(user_id, display_name)| json!({"userId": user_id, "displayName": display_name}))
        .collect();

    let params = [
        ("usersInfo", serde_json::to_string(&json_array)?),
        ("size", "HR64x64".to_string()),
    ];

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}

// Api: Emea v1
// Scope: https://ic3.teams.office.com/.default
pub async fn send_message(
    token: &AccessToken,
    conversation_id: String,
    body: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/conversations/{}/messages",
        conversation_id
    );
    if LOG_REQUESTS {
        println!("Log: GET {}", url);
    }
    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client.post(url).body(body).headers(headers).send().await?;

    if res.status().is_success() {
        let body = res.text().await?;
        Ok(body)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().await?
        );
        Err(error_message.into())
    }
}
