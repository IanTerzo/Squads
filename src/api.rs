use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub value: String,
    pub expires: u64,
}

#[derive(Debug, Clone)]
pub enum ApiError {
    RequestFailed(reqwest::StatusCode, String),
    MissingTokenOrExpiry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamSiteInformation {
    pub group_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub channels: Vec<Channel>,
    pub smtp_address: String,
    pub team_site_information: TeamSiteInformation,
    pub display_name: String,
    #[serde(deserialize_with = "trim_quotes")]
    pub picture_e_tag: String,
}

pub struct FileInfo {
    item_id: Option<String>,
    file_url: String,
    site_url: String,
    server_relative_url: String,
    share_url: Option<String>,
    share_id: Option<String>,
}

pub struct File {
    pub id: String,
    pub itemid: String,
    pub file_name: String,
    pub file_type: String,
    pub file_info: FileInfo,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmotionUser {
    pub mri: String,
    pub time: u64,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Emotion {
    pub key: String,
    pub users: Vec<EmotionUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageProperties {
    #[serde(default)]
    #[serde(deserialize_with = "string_to_i64")]
    pub edittime: i64,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub files: String, // is string that should be parsed to vec of File
    #[serde(default)]
    #[serde(deserialize_with = "string_to_i64")]
    pub deletetime: i64,
    #[serde(default)]
    #[serde(deserialize_with = "string_to_bool")]
    pub systemdelete: bool,
    pub title: Option<String>,
    pub emotions: Option<Vec<Emotion>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub content: Option<String>,
    pub from: Option<String>,
    pub im_display_name: Option<String>,
    pub message_type: Option<String>,
    pub properties: Option<MessageProperties>,
    pub compose_time: Option<String>,
    pub original_arrival_time: Option<String>,
    pub id: Option<String>,
    pub container_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub messages: Vec<Message>,
    pub container_id: String,
    pub id: String,
    pub latest_delivery_time: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamConversations {
    pub reply_chains: Vec<Conversation>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMember {
    pub is_muted: bool,
    pub mri: String,
    pub object_id: String,
    pub role: String,
    pub is_identity_masked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: String,
    pub title: Option<String>,
    pub last_message: Option<Message>,
    pub is_last_message_from_me: Option<bool>,
    pub chat_sub_type: Option<u64>,
    pub last_join_at: Option<String>,
    pub created_at: Option<String>,
    pub creator: String,
    pub members: Vec<ChatMember>,
    pub hidden: bool,
    pub added_by: Option<String>,
    pub chat_type: Option<String>,
    pub picture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDetails {
    pub teams: Vec<Team>,
    pub chats: Vec<Chat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShortProfile {
    pub user_principal_name: Option<String>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub user_location: Option<String>,
    pub email: Option<String>,
    pub user_type: Option<String>,
    pub is_short_profile: Option<bool>,
    pub tenant_name: Option<String>,
    pub company_name: Option<String>,
    pub display_name: Option<String>,
    pub r#type: Option<String>,
    pub mri: String,
    pub object_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FetchShortProfile {
    pub r#type: Option<String>,
    pub value: Vec<ShortProfile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProperties {
    pub is_skype_teams_user_set_in_settings_store: Option<String>,
    pub first_login_information: Option<String>,
    pub favorites: Option<String>,
    pub license_type: Option<String>,
    pub enable_channels_v2: Option<String>,
    pub personal_file_site: Option<String>,
    pub self_chat_settings: Option<String>,
    pub cortana_settings: Option<String>,
    pub teams_order: Option<String>,
    pub user_personal_settings: Option<String>,
    pub user_details: Option<String>,
    pub enable_push_to_talk: Option<String>,
    pub user_pinned_apps: Option<String>,
    pub locale: Option<String>,
    pub read_receipts_enabled: Option<String>,
    pub cid: Option<String>,
    pub cid_hex: Option<String>,
    pub dogfood_user: Option<bool>,
    pub primary_member_name: Option<String>,
    pub skype_name: Option<String>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::RequestFailed(status, body) => {
                write!(f, "Request failed with status {}: {}", status, body)
            }
            ApiError::MissingTokenOrExpiry => write!(f, "Missing refresh_token or expires_in"),
        }
    }
}

fn trim_quotes<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim_matches('"').to_string())
}
impl std::error::Error for ApiError {}

fn string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => i64::from_str(&s).map_err(serde::de::Error::custom),
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("Number is not a valid i64")),
        _ => Err(serde::de::Error::custom("Unexpected type")),
    }
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::String(s) => match s.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid boolean string: {}",
                s
            ))),
        },
        _ => Err(serde::de::Error::custom("Unexpected type")),
    }
}

fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn gen_refresh_token_from_code(
    code: String,
    code_verifier: String,
) -> Result<AccessToken, ApiError> {
    // Generate new refresh token if needed
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id=5e3ce6c0-2b1f-4285-8d4b-75ee78787346&\
    redirect_uri=https://teams.microsoft.com/v2&\
    scope=https://ic3.teams.office.com/.default openid profile offline_access&\
    code={}&\
    code_verifier={}&\
    grant_type=authorization_code&\
    claims={{\"access_token\":{{\"xms_cc\":{{\"values\":[\"CP1\"]}}}}}}",
        code, code_verifier
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .headers(headers)
        .body(body)
        .send()
        .unwrap();

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().unwrap();
        return Err(ApiError::RequestFailed(status, body));
    }

    let token_data: HashMap<String, Value> = res.json().unwrap();
    if let (Some(value), Some(expires_in)) = (
        token_data.get("refresh_token").and_then(|v| v.as_str()),
        token_data.get("expires_in").and_then(|v| v.as_u64()),
    ) {
        let refresh_token = AccessToken {
            value: value.to_string(),
            expires: get_epoch_s() + expires_in,
        };

        Ok(refresh_token)
    } else {
        Err(ApiError::MissingTokenOrExpiry)
    }
}

pub fn gen_tokens(refresh_token: AccessToken, scope: String) -> Result<AccessToken, ApiError> {
    // Generate new refresh token if needed
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id=5e3ce6c0-2b1f-4285-8d4b-75ee78787346&\
        scope={} openid profile offline_access&\
        grant_type=refresh_token&\
        client_info=1&\
        x-client-SKU=msal.js.browser&\
        x-client-VER=3.7.1&\
        refresh_token={}&\
        claims={{\"access_token\":{{\"xms_cc\":{{\"values\":[\"CP1\"]}}}}}}",
        scope, refresh_token.value
    );
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .headers(headers)
        .body(body)
        .send()
        .unwrap();

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().unwrap();
        return Err(ApiError::RequestFailed(status, body));
    }

    let token_data: HashMap<String, Value> = res.json().unwrap();
    if let (Some(value), Some(expires_in)) = (
        token_data.get("access_token").and_then(|v| v.as_str()),
        token_data.get("expires_in").and_then(|v| v.as_u64()),
    ) {
        let access_token = AccessToken {
            value: value.to_string(),
            expires: get_epoch_s() + expires_in,
        };
        Ok(access_token)
    } else {
        Err(ApiError::MissingTokenOrExpiry)
    }
}

async fn user_aggregate_settings(
    token: &AccessToken,
    json_body: HashMap<String, bool>,
) -> Result<HashMap<String, Value>, anyhow::Error> {
    //let scope = "https://api.spaces.skype.com/Authorization.ReadWrite".to_string();

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token)?,
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://teams.microsoft.com/api/mt/part/emea-02/beta/users/useraggregatesettings")
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

async fn gen_spoidcrl(
    token: AccessToken,
    section: String,
    web_url: Value,
) -> Result<AccessToken, anyhow::Error> {
    //let scope = format!("{}/.default", web_url.to_string().replace('\"', ""));

    let access_token = format!("Bearer {}", token.value);

    let url = format!(
        "{}/sites/{}/_api/SP.OAuth.NativeClient/Authenticate",
        web_url.to_string().replace('\"', ""),
        section
    );

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
pub fn gen_skype_token(token: AccessToken) -> Result<AccessToken, anyhow::Error> {
    //let req_scope = "https://api.spaces.skype.com/Authorization.ReadWrite".to_string();

    let req_access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&req_access_token)?,
    );
    headers.insert("Content-Length", "0".parse()?);

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let res = client
        .post("https://teams.microsoft.com/api/authsvc/v1.0/authz")
        .headers(headers)
        .send()?;

    if res.status().is_success() {
        let token_data: HashMap<String, Value> = res.json()?;

        if let Some(tokens) = token_data.get("tokens").and_then(|v| v.as_object()) {
            if let (Some(value), Some(expires_in)) = (
                tokens.get("skypeToken").and_then(|v| v.as_str()),
                tokens.get("expiresIn").and_then(|v| v.as_u64()),
            ) {
                let access_token = AccessToken {
                    value: value.to_string(),
                    expires: get_epoch_s() + expires_in,
                };

                Ok(access_token)
            } else {
                Err(anyhow!("Couldn't get response skypeToken or expiresIn"))
            }
        } else {
            Err(anyhow!("Couldn't get response tokens"))
        }
    } else {
        let status = res.status();
        let body = res.text()?;
        Err(anyhow!("{}: {}", status, body))
    }
}

pub fn user_details(token: AccessToken) -> Result<UserDetails, String> {
    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let params = [
        ("isPrefetch", "false"),
        ("enableMembershipSummary", "true"),
        ("enableRC2Fetch", "false"),
    ];

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get("https://teams.microsoft.com/api/csa/emea/api/v2/teams/users/me")
        .headers(headers)
        .query(&params)
        .send()
        .unwrap();

    if res.status().is_success() {
        let body = res.text().unwrap();
        let parsed_body: Value = serde_json::from_str(&body).expect("Invalid JSON");
        let pretty_json =
            serde_json::to_string_pretty(&parsed_body).expect("Failed to format JSON");
        let result: Result<UserDetails, serde_json::Error> = serde_json::from_str(&pretty_json);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                println!("Error occurred while serializing: {}", err);
                let line_content = pretty_json.lines().nth(err.line() - 1).unwrap();
                println!("Line: {}", line_content);

                Err(err.to_string())
            }
        }
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

pub fn user_properties(token: AccessToken) -> Result<UserProperties, String> {
    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get("https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/properties")
        .headers(headers)
        .send()
        .unwrap();

    if res.status().is_success() {
        let parsed = res.json::<UserProperties>().unwrap();
        Ok(parsed)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

pub fn fetch_short_profile(
    token: AccessToken,
    user_ids: Vec<String>,
) -> Result<FetchShortProfile, String> {
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

    let client = reqwest::blocking::Client::builder()
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
        .post("https://teams.microsoft.com/api/mt/part/emea-02/beta/users/fetchShortProfile")
        .headers(headers)
        .query(&params)
        .body(body)
        .send()
        .unwrap();

    if res.status().is_success() {
        let parsed = res.json::<FetchShortProfile>().unwrap();
        Ok(parsed)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

pub fn team_conversations(
    token: AccessToken,
    team_id: String,
    topic_id: String,
) -> Result<TeamConversations, String> {
    //let scope = "https://chatsvcagg.teams.microsoft.com/.default".to_string();

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get(format!(
            "https://teams.microsoft.com/api/csa/emea/api/v2/teams/{}/channels/{}",
            team_id, topic_id
        ))
        .headers(headers)
        .send()
        .unwrap();

    if res.status().is_success() {
        let body = res.text().unwrap();
        let parsed: TeamConversations = serde_json::from_str(&body).unwrap();
        Ok(parsed)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

async fn team_channel_info(
    token: AccessToken,
    group_id: String,
    topic_id: String,
) -> Result<HashMap<String, Value>, String> {
    //let scope = "https://graph.microsoft.com/.default".to_string();

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let url = format!(
        "https://graph.microsoft.com/beta/teams/{}/channels/{}",
        group_id, topic_id
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

async fn document_libraries(
    token: AccessToken,
    skype_token: AccessToken,
    topic_id: String,
) -> Result<Vec<Value>, String> {
    //let mut scope = "https://api.spaces.skype.com/.default".to_string();

    let access_token = format!("Bearer {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );
    headers.insert("x-skypetoken", skype_token.value.parse().unwrap());

    let url = format!(
        "https://teams.microsoft.com/api/mt/part/emea-02/beta/channels/{}/documentlibraries",
        topic_id
    );
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

async fn render_list_data_as_stream(
    token: AccessToken,
    web_url: String,
    section: String,
    files_relative_path: String,
) -> Result<HashMap<String, Value>, String> {
    //let scope = "spoidcrl".to_string();

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

    let url = format!(
        "{}/sites/{}/_api/web/GetListUsingPath(DecodedUrl=@a1)/RenderListDataAsStream?@a1='{}'&RootFolder={}&TryNewExperienceSingle=TRUE",
        web_url.to_string().replace('\"', ""),
        section,
        files_relative_path,
        files_relative_path
    );

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

pub fn authorize_team_picture(
    token: AccessToken,
    group_id: String,
    etag: String,
    display_name: String,
) -> Result<Bytes, String> {
    //let scope = "https://api.spaces.skype.com/Authorization.ReadWrite".to_string();

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
        )
        .unwrap(),
    );

    let params = [("etag", etag), ("displayName", display_name)];

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get(format!(
            "https://teams.microsoft.com/api/mt/part/emea-02/beta/users/15de4241-e9be-4910-a60f-3f37dd8652b8/profilepicturev2/teams/{}",
            group_id
        ))
        .headers(headers)
        .query(&params)
        .send()
        .unwrap();

    if res.status().is_success() {
        let bytes = res.bytes().unwrap();
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

pub fn authorize_profile_picture(
    token: AccessToken,
    user_id: String,
    display_name: String,
) -> Result<Bytes, String> {
    // let scope = "https://api.spaces.skype.com/Authorization.ReadWrite".to_string();

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
        )
        .unwrap(),
    );

    let params = [
        ("displayname", display_name),
        ("size", "HR64x64".to_string()),
    ];

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get(format!(
            "https://teams.microsoft.com/api/mt/part/emea-02/beta/users/{}/profilepicturev2",
            user_id
        ))
        .headers(headers)
        .query(&params)
        .send()
        .unwrap();

    if res.status().is_success() {
        let bytes = res.bytes().unwrap();
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

pub fn authorize_image(token: AccessToken, image_id: String) -> Result<Bytes, String> {
    //let scope = "skype_token".to_string();

    let access_token = format!("skype_token {}", token.value);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&access_token).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get(format!(
            "https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/{}/views/imgo?v=1",
            image_id
        ))
        .headers(headers)
        .send()
        .unwrap();

    if res.status().is_success() {
        let bytes = res.bytes().unwrap();
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}

pub fn authorize_avatar(token: AccessToken, avatar_url: String) -> Result<Bytes, String> {
    // let scope = "https://api.spaces.skype.com/Authorization.ReadWrite".to_string();

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("referer"),
        HeaderValue::from_static("https://teams.microsoft.com/_"),
    );
    headers.insert(
        HeaderName::from_static("cookie"),
        HeaderValue::from_str(format!("skypetoken_asm={}", token.value).as_str()).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client.get(avatar_url).headers(headers).send().unwrap();

    if res.status().is_success() {
        let bytes = res.bytes().unwrap();
        Ok(bytes)
    } else {
        let error_message = format!(
            "Status code: {}, Response body: {}",
            res.status(),
            res.text().unwrap()
        );
        Err(error_message)
    }
}
