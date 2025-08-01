use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum MessageAreaAction {
    Bold,
    Italic,
    Underline,
    Striketrough,
    Code,
    Blockquote,
    Link,
    Image,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TeamsMessage<'a> {
    pub id: &'a str,
    #[serde(rename = "type")]
    pub msg_type: &'a str,
    pub conversationid: &'a str,
    pub conversation_link: &'a str,
    pub from: &'a str,
    pub composetime: &'a str,
    pub originalarrivaltime: &'a str,
    pub content: &'a str,
    pub messagetype: &'a str,
    pub contenttype: &'a str,
    pub imdisplayname: &'a str,
    pub clientmessageid: &'a str,
    pub call_id: &'a str,
    pub state: i32,
    pub version: &'a str,
    pub amsreferences: Vec<&'a str>,
    pub properties: Properties<'a>,
    pub post_type: &'a str,
    pub cross_post_channels: Vec<&'a str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Properties<'a> {
    pub importance: &'a str,
    pub subject: Option<&'a str>,
    pub title: &'a str,
    pub cards: &'a str,
    pub links: &'a str,
    pub mentions: &'a str,
    pub onbehalfof: Option<&'a str>,
    pub files: &'a str,
    pub policy_violation: Option<&'a str>,
    pub format_variant: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMember {
    pub id: String,
    pub role: String,
    pub share_history_time: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadProperties {
    pub thread_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thread {
    pub members: Vec<ThreadMember>,
    pub properties: Option<ThreadProperties>,
}
