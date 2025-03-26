use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TeamsMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub conversationid: String,
    pub conversation_link: String,
    pub from: String,
    pub composetime: String,
    pub originalarrivaltime: String,
    pub content: String,
    pub messagetype: String,
    pub contenttype: String,
    pub imdisplayname: String,
    pub clientmessageid: String,
    pub call_id: String,
    pub state: i32,
    pub version: String,
    pub amsreferences: Vec<String>,
    pub properties: Properties,
    pub post_type: String,
    pub cross_post_channels: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub importance: String,
    pub subject: String,
    pub title: String,
    pub cards: String,
    pub links: String,
    pub mentions: String,
    pub onbehalfof: Option<String>,
    pub files: String,
    pub policy_violation: Option<String>,
    pub format_variant: String,
}
