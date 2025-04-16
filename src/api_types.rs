use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCodeInfo {
    #[serde(rename = "user_code")]
    pub user_code: String,
    #[serde(rename = "device_code")]
    pub device_code: String,
    #[serde(rename = "verification_url")]
    pub verification_url: String,
    #[serde(rename = "expires_in")]
    pub expires_in: String,
    pub interval: String,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResponse {
    pub scope: String,
    pub foci: u64,
    pub refresh_token: String,
    pub token_type: String,
    pub not_before: u64,
    pub expires_on: u64,
    pub resource: String,
    pub expires_in: u64,
    pub id_token: String,
    pub ext_expires_in: u64,
    pub access_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub value: String,
    pub expires: u64,
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
    pub smtp_address: Option<String>,
    pub team_site_information: TeamSiteInformation,
    pub display_name: String,
    #[serde(deserialize_with = "trim_quotes")]
    pub picture_e_tag: Option<String>, // In some small cases this is not set
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmotionUser {
    pub mri: String,
    pub time: u64,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Emotion {
    pub key: String,
    pub users: Vec<EmotionUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityContext {
    pub teams_app_id: Option<String>,
    pub location: Option<String>,
    pub template_parameter: Option<String>,
    pub entitlement_search_locations: Option<String>,
    pub attributed_to_actor: Option<String>,
    pub attributed_to_actor_id: Option<String>,
    #[serde(rename = "AggregationId")]
    pub aggregation_id: Option<String>,
    #[serde(rename = "WebhookCorrelationId")]
    pub webhook_correlation_id: Option<String>,
    #[serde(rename = "ClumpId")]
    pub clump_id: Option<String>,
    #[serde(rename = "ClumpType")]
    pub clump_type: Option<String>,
    #[serde(rename = "ClumpTitle")]
    pub clump_title: Option<String>,
    pub activity_version: Option<String>,
    pub activity_processing_latency: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub activity_type: String,
    pub activity_subtype: Option<String>,
    pub activity_timestamp: String,
    pub activity_id: u64,
    pub source_message_id: u64,
    pub source_reply_chain_id: Option<u64>,
    pub source_user_id: String,
    pub source_user_im_display_name: Option<String>,
    pub target_user_id: String,
    pub source_thread_id: String,
    pub message_preview: String,
    pub message_preview_template_option: String,
    pub source_thread_topic: Option<String>,
    pub source_thread_roster_non_bot_member_count: Option<u64>,
    #[serde(deserialize_with = "string_to_bool")]
    pub source_thread_is_private_channel: bool,
    pub activityContext: ActivityContext,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProperties {
    #[serde(default)]
    #[serde(deserialize_with = "string_to_i64")]
    pub edittime: i64,
    pub subject: Option<String>,
    #[serde(default)]
    pub files: Option<String>, // is string that should be parsed to vec of File
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_cards")]
    pub cards: Option<Vec<Card>>,
    #[serde(default)]
    #[serde(deserialize_with = "string_to_i64")]
    pub deletetime: i64,
    #[serde(default)]
    #[serde(deserialize_with = "string_to_bool")]
    pub systemdelete: bool,
    pub title: Option<String>,
    pub emotions: Option<Vec<Emotion>>,
    #[serde(rename = "isread")]
    #[serde(default)]
    #[serde(deserialize_with = "string_to_option_bool")]
    pub is_read: Option<bool>,
    pub activity: Option<Activity>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardContentButton {
    #[serde(rename = "type")]
    pub button_type: String,
    pub title: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardContent {
    pub text: Option<String>,
    pub component_url: Option<String>,
    pub source_type: Option<String>,
    pub buttons: Option<Vec<CardContentButton>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub app_icon: Option<String>,
    pub card_client_id: String,
    pub content: CardContent,
    pub content_type: String,
    pub preview_hidden: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub content: Option<String>,
    #[serde(deserialize_with = "strip_url")]
    // In some cases the id is displayed as a contacts url, https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/contacts/8:orgid:15de4241-3...
    pub from: Option<String>,
    #[serde(alias = "imdisplayname")]
    pub im_display_name: Option<String>,
    #[serde(alias = "messagetype")]
    pub message_type: Option<String>,
    pub properties: Option<MessageProperties>,
    pub compose_time: Option<String>,
    #[serde(alias = "originalarrivaltime")]
    pub original_arrival_time: Option<String>,
    pub conversation_link: Option<String>,
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
pub struct Conversations {
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamConversations {
    pub reply_chains: Vec<Conversation>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMember {
    pub is_muted: Option<bool>,
    pub mri: String,
    pub object_id: Option<String>,
    pub role: Option<String>,
    pub is_identity_masked: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: String,
    pub members: Vec<ChatMember>,
    pub is_read: Option<bool>,
    pub is_high_importance: Option<bool>,
    pub is_one_on_one: Option<bool>,
    pub is_conversation_deleted: Option<bool>,
    pub is_external: Option<bool>,
    pub is_messaging_disabled: Option<bool>,
    pub is_disabled: Option<bool>,
    pub title: Option<String>,
    pub last_message: Option<Message>,
    pub is_last_message_from_me: Option<bool>,
    pub chat_sub_type: Option<u64>,
    pub last_join_at: Option<String>,
    pub created_at: Option<String>,
    pub creator: Option<String>,
    pub hidden: Option<bool>,
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
#[derive(Default)]
pub struct Profile {
    pub id: String,
    pub display_name: Option<String>,
    pub business_phones: Option<Vec<String>>,
    pub given_name: Option<String>,
    pub job_title: Option<String>,
    pub mail: Option<String>,
    pub mobile_phone: Option<String>,
    pub office_location: Option<String>,
    pub preferred_language: Option<String>,
    pub is_short_profile: Option<bool>,
    pub surname: Option<String>,
    pub company_name: Option<String>,
    pub user_principal_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Users {
    pub value: Vec<Profile>,
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

pub fn strip_url<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(|url| {
        let pass1 = url
            .strip_prefix("https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/contacts/")
            .unwrap_or(&url);
        pass1
            .strip_prefix("https://notifications.skype.net/v1/users/ME/contacts/")
            .unwrap_or(&pass1)
            .to_string()
    }))
}

fn trim_quotes<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.map(|s| s.trim_matches('"').to_string()))
}

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

pub fn string_to_option_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    if let Some(value) = value {
        match value {
            serde_json::Value::Bool(b) => Ok(Some(b)),
            serde_json::Value::String(s) => match s.as_str() {
                "true" => Ok(Some(true)),
                "false" => Ok(Some(false)),
                _ => Err(serde::de::Error::custom(format!(
                    "Invalid boolean string: {}",
                    s
                ))),
            },
            _ => Err(serde::de::Error::custom("Unexpected type")),
        }
    } else {
        Ok(None)
    }
}

fn deserialize_cards<'de, D>(deserializer: D) -> Result<Option<Vec<Card>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(json_str) => serde_json::from_str(&json_str)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}
