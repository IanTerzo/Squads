mod api;
mod api_types;
mod components;
mod parsing;
use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use iced::task::Handle;
use iced::widget::text_input::{self, focus};
use itertools::Itertools;
use parsing::parse_message_markdown;
mod auth;
mod pages;
mod style;
mod types;
mod utils;
mod websockets;
mod widgets;
use api::{
    authorize_image, authorize_merged_profile_picture, authorize_profile_picture,
    authorize_team_picture, consumption_horizon, conversations, gen_refresh_token_from_device_code,
    me, send_message, sharepoint_download_file, site_info, team_conversations, teams_me, users,
    AccessToken, Chat, Conversations, DeviceCodeInfo, File, Profile, Team, TeamConversations,
};
use auth::{get_or_gen_skype_token, get_or_gen_token};
use components::{cached_image::save_cached_image, expanded_image::c_expanded_image};
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::widget::scrollable::{snap_to, Id, RelativeOffset, Viewport};
use iced::widget::text_editor::{self, Action, Content, Edit};
use iced::{
    event, keyboard, mouse, window, Color, Element, Event, Font, Padding, Point, Size,
    Subscription, Task, Theme,
};
use pages::app;
use pages::page_chat::chat;
use pages::page_home::home;
use pages::page_login::login;
use pages::page_team::team;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{collections::HashMap, fs};
use std::{env, fmt::format, thread};
use style::global_theme;
use tokio::time::sleep;
use types::*;
use utils::{get_cache, get_epoch_ms, save_to_cache};
use webbrowser;
use websockets::{
    connect, websockets_subscription, ConnectionInfo, Presence, Presences, WebsocketMessage,
    WebsocketResponse,
};

use crate::api::{add_member, emotions, is_read, start_thread, ChatMember, Conversation};
use crate::components::emoji_picker::{
    self, c_emoji_picker, EmojiPickerAlignment, EmojiPickerPosition,
};
use crate::parsing::get_html_preview;

const WINDOW_WIDTH: f32 = 1240.0;
const WINDOW_HEIGHT: f32 = 780.0;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ChatBody {
    Messages,
    Members,
    Add,
    Start,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
enum View {
    Login,
    Homepage,
    Team,
    Chat,
}

// Any information needed to display the current page
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Page {
    view: View,
    chat_body: ChatBody,
    current_team_id: Option<String>,
    current_channel_id: Option<String>,
    current_chat_id: Option<String>,
}

#[derive(Debug)]
struct Counter {
    // Authorization info
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    is_authorized: bool,
    device_code: String, // Only used when signing in for the first time
    device_user_code: Option<String>, // Only used when signing in for the first time
    tenant: String,

    // App info
    page: Page,
    history: Vec<Page>,
    history_index: usize,
    theme: style::Theme,
    emoji_map: HashMap<String, String>,
    window_width: f32,
    window_height: f32,
    mouse_position: (f32, f32),
    last_mouse_position: (f32, f32),
    shift_held_down: bool,
    scrollbar_scroll: u64,
    scrollbar_percentage_scroll: f32,
    should_send_typing: bool,
    users_typing_timeouts: HashMap<String, HashMap<String, Handle>>, // Where string is the chat id and the other string is the user id
    emoji_picker_hide_options: Option<String>,

    // UI state
    reply_options: HashMap<String, bool>, // String is the conversation id
    chat_message_options: HashMap<String, bool>, // String is the message id
    team_conversations: HashMap<String, TeamConversations>, // String is the team id
    chat_conversations: HashMap<String, Vec<api::Message>>, // String is the thread id
    activity_expanded_conversations: HashMap<String, (bool, Vec<api::Message>)>, // String is the thread id, bool is toggled
    search_teams_input_value: String,
    search_chats_input_value: String,
    search_users_input_value: String,
    team_message_area_content: Content,
    team_message_area_height: f32,
    chat_message_area_content: Content,
    chat_message_area_height: f32,
    subject_input_value: String,
    expanded_image: Option<(String, String)>,
    add_users_checked: HashMap<String, bool>, // Where string is the user id
    emoji_picker_toggle: EmojiPickerInfo,

    // Teams requested data
    me: Profile,
    users: HashMap<String, Profile>,
    user_presences: HashMap<String, Presence>, // Where string is the user id
    teams: Vec<Team>,
    chats: Vec<Chat>,
    activities: Vec<api::Message>,
    websockets_connection_info: Option<ConnectionInfo>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Authorization
    GotDeviceCodeInfo(DeviceCodeInfo),
    PollDeviceCode,
    Authorized(AccessToken),

    // App actions
    EventOccurred(Event),
    ToggleShift(bool),
    KeyPressed(Key),

    // UI interactions
    MessageAreaEdit(text_editor::Action),
    MessageAreaAction(MessageAreaAction),
    LinkClicked(String),
    OpenHome,
    OpenTeam(String, String),
    OpenChat(String),
    ReadChat(String),
    OpenCurrentChat,
    ToggleReplyOptions(String),
    ShowChatMessageOptions(String),
    StopShowChatMessageOptions(String),
    HistoryBack,
    HistoryForward,
    ExpandImage(String, String),
    StopExpandImage,
    SearchTeamsContentChanged(String),
    SearchChatsContentChanged(String),
    SearchUsersContentChanged(String),
    SubjectInputContentChanged(String),
    AllowPostIsTyping(()),
    ToggleNewChatMenu,
    Reply(Option<String>, Option<String>, Option<String>),
    // Teams requests
    GotActivities(Vec<api::Message>),
    GotUsers(HashMap<String, Profile>, Profile),
    GotUserDetails(Vec<Team>, Vec<Chat>),
    // UI initiated
    ToggleExpandActivity(String, u64, String),
    GotExpandedActivity(String, Vec<api::Message>), //callback
    PrefetchChat(String),
    PrefetchCurrentChat,
    GotChatConversations(String, Conversations), //callback
    PrefetchTeam(String, String),
    GotConversations(String, TeamConversations), //callback
    OnScroll(Viewport),
    PostMessage,
    StartChat(Vec<String>),
    CreatedGroupChat(String, String, String), //callback
    AddToGroupChat(String, Vec<String>),
    AddedToGroupChat(String, Vec<String>),
    FetchTeamImage(String, String, String, String),
    FetchUserImage(String, String, String),
    FetchMergedProfilePicture(String, Vec<(String, String)>),
    AuthorizeImage(String, String),
    DownloadImage(String, String),
    DownloadFile(File),
    DownloadedFile(String),
    ToggleShowChatMembers,
    ToggleShowChatAdd,
    ToggleUserCheckbox(bool, String),
    ToggleEmojiPicker(Option<EmojiPickerLocation>, EmojiPickerAction),
    EmojiPickerPicked(String, String),

    // Websockets
    WSConnected(ConnectionInfo),
    GotWSMessage(WebsocketMessage),
    GotWSPresences(Presences),
    TypingTimeoutFinished(String, String),

    // Other
    DoNothing(()),
    Join,          // For testing
    Hello(String), // For testing
}

fn init_tasks(
    access_tokens: std::sync::Arc<RwLock<HashMap<String, AccessToken>>>,
    tenant: String,
) -> Task<Message> {
    let access_tokens1 = Arc::clone(&access_tokens);
    let access_tokens2 = Arc::clone(&access_tokens);
    let access_tokens3 = Arc::clone(&access_tokens);

    let tenant1 = tenant.clone();
    let tenant2 = tenant.clone();

    Task::batch(vec![
        Task::perform(
            async move {
                let access_token_ic3 = get_or_gen_token(
                    access_tokens1,
                    "https://ic3.teams.office.com/.default".to_string(),
                    &tenant,
                )
                .await;
                let activity_messages =
                    conversations(&access_token_ic3, "48:notifications".to_string(), None)
                        .await
                        .unwrap();

                activity_messages.messages
            },
            Message::GotActivities,
        ),
        Task::perform(
            async move {
                let access_token_chatsvcagg = get_or_gen_token(
                    access_tokens2,
                    "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                    &tenant1,
                )
                .await;

                let user_details = teams_me(&access_token_chatsvcagg).await.unwrap();

                let teams = user_details.teams;
                let chats = user_details.chats;

                save_to_cache("teams.json", &teams);
                save_to_cache("chats.json", &chats);

                (teams, chats)
            },
            |result| Message::GotUserDetails(result.0, result.1),
        ),
        Task::perform(
            async move {
                let access_token_graph = get_or_gen_token(
                    access_tokens3,
                    "https://graph.microsoft.com/.default".to_string(),
                    &tenant2,
                )
                .await;

                let mut profile_map = HashMap::new();
                let mut next_link_params = Some("$top=555".to_string());

                // Iterate while there is a next_link

                while let Some(params) = next_link_params.take() {
                    let users_value = users(&access_token_graph, &params).await.unwrap();

                    for profile in users_value.value {
                        profile_map.insert(profile.id.clone(), profile);
                    }

                    next_link_params = users_value
                        .next_link
                        .map(|url| url.replace("https://graph.microsoft.com/v1.0/users?", ""));
                }

                let profile = me(&access_token_graph).await.unwrap();

                save_to_cache("users.json", &profile_map.to_owned());
                save_to_cache("me.json", &profile.to_owned());

                (profile_map, profile)
            },
            |result| Message::GotUsers(result.0, result.1),
        ),
    ])
}

fn post_message_task(
    message_area_text: String,
    acess_tokens_arc: Arc<RwLock<HashMap<String, AccessToken>>>,
    tenant: String,
    conversation_id: String,
    me_id: String,
    me_display_name: Option<String>,
    subject: Option<String>,
) -> Task<Message> {
    Task::perform(
        async move {
            let html = parse_message_markdown(message_area_text);

            let access_token = get_or_gen_token(
                acess_tokens_arc,
                "https://ic3.teams.office.com/.default".to_string(),
                &tenant,
            )
            .await;

            let mut rng = StdRng::from_os_rng();
            let message_id: u64 = rng.random(); // generate the message_id randomly

            let message = TeamsMessage {
                id: "-1",
                msg_type: "Message",
                conversationid: &conversation_id,
                conversation_link: &format!("blah/{}", conversation_id),
                from: &format!("8:orgid:{}", me_id),
                composetime: "2025-03-06T11:04:18.265Z",
                originalarrivaltime: "2025-03-06T11:04:18.265Z",
                content: &html,
                messagetype: "RichText/Html",
                contenttype: "Text",
                imdisplayname: me_display_name.as_deref(),
                clientmessageid: &message_id.to_string(),
                call_id: "",
                state: 0,
                version: "0",
                amsreferences: vec![],
                properties: Properties {
                    importance: "",
                    subject: subject.as_deref(),
                    title: "",
                    cards: "[]",
                    links: "[]",
                    mentions: "[]",
                    onbehalfof: None,
                    files: "[]",
                    policy_violation: None,
                    format_variant: "TEAMS",
                },
                post_type: "Standard",
                cross_post_channels: vec![],
            };

            // Convert the struct into a JSON string
            let body = serde_json::to_string_pretty(&message).unwrap();

            send_message(&access_token, conversation_id, body)
                .await
                .unwrap();
        },
        Message::DoNothing,
    )
}

fn content_send(content: &mut Content, message: &str) {
    for char in message.chars() {
        content.perform(Action::Edit(Edit::Insert(char)));
    }
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        let file_content =
            fs::read_to_string(utils::get_resource_dir().join("emojis.json")).unwrap();
        let emojis: HashMap<String, String> = serde_json::from_str(&file_content).unwrap();

        let access_tokens = Arc::new(RwLock::new(HashMap::new()));
        if let Some(cached) = get_cache::<HashMap<String, AccessToken>>("access_tokens.json") {
            *access_tokens.write().unwrap() = cached;
        }
        let teams = get_cache::<Vec<Team>>("teams.json").unwrap_or(Vec::new());
        let chats = get_cache::<Vec<Chat>>("chats.json").unwrap_or(Vec::new());
        let user_profiles =
            get_cache::<HashMap<String, Profile>>("users.json").unwrap_or(HashMap::new());
        let profile = get_cache::<Profile>("me.json").unwrap_or(Profile::default());

        // If the user doesn't have a refresh token, prompt them to the login page.
        let has_refresh_token = access_tokens.read().unwrap().get("refresh_token").is_some();

        let tenant = "organizations".to_string();

        let first_chat = chats.get(0).map(|chat| chat.id.clone());

        let counter_self = Self {
            page: Page {
                view: if has_refresh_token {
                    View::Homepage
                } else {
                    View::Login
                },
                chat_body: ChatBody::Messages,
                current_team_id: None,
                current_channel_id: None,
                current_chat_id: first_chat.clone(),
            },
            theme: global_theme(),
            device_user_code: None,
            device_code: "".to_string(),
            tenant: tenant.clone(),
            is_authorized: has_refresh_token,
            team_message_area_content: Content::new(),
            team_message_area_height: 54.0,
            chat_message_area_content: Content::new(),
            chat_message_area_height: 54.0,
            reply_options: HashMap::new(),
            scrollbar_scroll: 0,
            scrollbar_percentage_scroll: 1.0,
            chat_message_options: HashMap::new(),
            users_typing_timeouts: HashMap::new(),
            history: vec![Page {
                view: View::Homepage,
                chat_body: ChatBody::Messages,
                current_team_id: None,
                current_channel_id: None,
                current_chat_id: first_chat,
            }],
            websockets_connection_info: None,
            user_presences: HashMap::new(),
            add_users_checked: HashMap::new(),
            expanded_image: None,
            should_send_typing: true,
            history_index: 0,
            emoji_map: emojis,
            search_teams_input_value: "".to_string(),
            search_chats_input_value: "".to_string(),
            search_users_input_value: "".to_string(),
            subject_input_value: "".to_string(),
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            mouse_position: (0.0, 0.0),
            last_mouse_position: (0.0, 0.0),
            access_tokens: access_tokens.clone(),
            users: user_profiles,
            me: profile,
            teams: teams.clone(),
            chats: chats.clone(),
            activity_expanded_conversations: HashMap::new(),
            team_conversations: HashMap::new(),
            chat_conversations: HashMap::new(),
            activities: Vec::new(),
            shift_held_down: false,
            emoji_picker_toggle: EmojiPickerInfo {
                action: EmojiPickerAction::None,
                location: None,
            },
            emoji_picker_hide_options: None,
        };
        (
            counter_self,
            if has_refresh_token {
                init_tasks(access_tokens, tenant)
            } else {
                Task::perform(
                    async move { api::gen_device_code(tenant).await.unwrap() },
                    Message::GotDeviceCodeInfo,
                )
            },
        )
    }

    fn view(&self) -> Element<Message> {
        //println!("view called");

        match self.page.view {
            View::Login => login(&self.theme, &self.device_user_code),
            View::Homepage => app(
                &self.theme,
                home(
                    &self.theme,
                    &self.teams,
                    &self.activities,
                    self.activity_expanded_conversations.clone(),
                    &self.emoji_map,
                    &self.users,
                    &self.user_presences,
                    self.window_width,
                    self.search_teams_input_value.clone(),
                ),
                if let Some(expanded_image) = self.expanded_image.clone() {
                    Some(c_expanded_image(expanded_image.0, expanded_image.1))
                } else {
                    None
                },
            ),
            View::Team => {
                let current_team_id = self.page.current_team_id.as_ref().unwrap();

                let mut current_team = self
                    .teams
                    .iter()
                    .find(|team| &team.id == current_team_id)
                    .unwrap()
                    .clone();

                let current_channel_id = self.page.current_channel_id.as_ref().unwrap();

                let current_channel = current_team
                    .channels
                    .iter()
                    .find(|channel| &channel.id == current_channel_id)
                    .unwrap()
                    .clone();

                let reply_options = &self.reply_options;

                let conversation = self.team_conversations.get(current_channel_id);

                app(
                    &self.theme,
                    team(
                        &self.theme,
                        &mut current_team,
                        &current_channel,
                        &conversation,
                        &reply_options,
                        &self.emoji_map,
                        &self.users,
                        &self.user_presences,
                        &self.subject_input_value,
                        &self.team_message_area_content,
                        &self.team_message_area_height,
                    ),
                    if let Some(expanded_image) = self.expanded_image.clone() {
                        Some(c_expanded_image(expanded_image.0, expanded_image.1))
                    } else {
                        None
                    },
                )
            }
            View::Chat => {
                let current_chat = if let Some(current_chat_id) = &self.page.current_chat_id {
                    Some(
                        self.chats
                            .iter()
                            .find(|chat| &chat.id == current_chat_id)
                            .unwrap(),
                    )
                } else {
                    None
                };

                let conversation = if let Some(current_chat_id) = &self.page.current_chat_id {
                    self.chat_conversations.get(current_chat_id)
                } else {
                    None
                };

                app(
                    &self.theme,
                    chat(
                        &self.theme,
                        current_chat,
                        &self.users_typing_timeouts,
                        &self.add_users_checked,
                        &self.chats,
                        &conversation,
                        &self.chat_message_options,
                        &self.emoji_map,
                        &self.users,
                        &self.user_presences,
                        &self.me,
                        self.search_chats_input_value.clone(),
                        self.search_users_input_value.clone(),
                        &self.chat_message_area_content,
                        &self.chat_message_area_height,
                        &self.page.chat_body,
                    ),
                    if let Some(expanded_image) = self.expanded_image.clone() {
                        Some(c_expanded_image(expanded_image.0, expanded_image.1))
                    } else if self.emoji_picker_toggle.action != EmojiPickerAction::None {
                        if let Some(ref location) = self.emoji_picker_toggle.location {
                            Some(c_emoji_picker(
                                &self.theme,
                                match location {
                                    EmojiPickerLocation::OverMessageArea => EmojiPickerPosition {
                                        alignment: EmojiPickerAlignment::BottomLeft,
                                        padding: Padding {
                                            top: 0.0,
                                            right: 0.0,
                                            bottom: 150.0,
                                            left: 260.0,
                                        },
                                    },
                                    EmojiPickerLocation::ReactionContext => EmojiPickerPosition {
                                        alignment: EmojiPickerAlignment::TopRight,
                                        padding: Padding {
                                            top: if self.last_mouse_position.1 - 30.0
                                                < self.window_height - 450.0
                                            {
                                                self.last_mouse_position.1 - 30.0
                                            } else {
                                                self.window_height - 450.0
                                            },
                                            right: 84.0,
                                            bottom: 0.0,
                                            left: 0.0,
                                        },
                                    },
                                    EmojiPickerLocation::ReactionAdd => EmojiPickerPosition {
                                        alignment: EmojiPickerAlignment::TopLeft,
                                        padding: Padding {
                                            top: if self.last_mouse_position.1 - 30.0
                                                < self.window_height - 450.0
                                            {
                                                self.last_mouse_position.1 - 30.0
                                            } else {
                                                self.window_height - 450.0
                                            },
                                            right: 0.0,
                                            bottom: 0.0,
                                            left: 365.0,
                                        },
                                    },
                                },
                                &self.emoji_map,
                            ))
                        } else {
                            None // Should never happen
                        }
                    } else {
                        None
                    },
                )
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Authorization
            Message::GotDeviceCodeInfo(device_code_info) => {
                self.device_user_code = Some(device_code_info.user_code);
                self.device_code = device_code_info.device_code.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async {
                        thread::sleep(Duration::new(1, 0));

                        let mut refresh_token: Option<AccessToken> = None;
                        let result = gen_refresh_token_from_device_code(
                            device_code_info.device_code,
                            tenant,
                        )
                        .await;
                        if let Ok(access_token) = result {
                            refresh_token = Some(access_token);
                            println!("Code polling succeeded.")
                        }

                        refresh_token
                    },
                    |refresh_token| {
                        if let Some(refresh_token) = refresh_token {
                            Message::Authorized(refresh_token)
                        } else {
                            Message::PollDeviceCode
                        }
                    },
                )
            }
            Message::PollDeviceCode => {
                let device_code = self.device_code.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async {
                        thread::sleep(Duration::new(1, 0));

                        let mut refresh_token: Option<AccessToken> = None;
                        let result = gen_refresh_token_from_device_code(device_code, tenant).await;
                        // TODO: log non re-poll errors
                        if let Ok(access_token) = result {
                            refresh_token = Some(access_token);
                            println!("Code polling succeeded.")
                        }

                        refresh_token
                    },
                    |refresh_token| {
                        if let Some(refresh_token) = refresh_token {
                            Message::Authorized(refresh_token)
                        } else {
                            Message::PollDeviceCode
                        }
                    },
                )
            }
            Message::Authorized(refresh_token) => {
                self.page.view = View::Homepage;

                self.access_tokens
                    .write()
                    .unwrap()
                    .insert("refresh_token".to_string(), refresh_token);
                self.is_authorized = true;
                init_tasks(self.access_tokens.clone(), self.tenant.clone())
            }

            // App actions
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(window::Event::Resized(size)) => {
                        self.window_width = size.width;
                        self.window_height = size.height;

                        let max_area_height = 0.5 * self.window_height;

                        if self.team_message_area_height > max_area_height {
                            self.team_message_area_height = max_area_height;
                        }

                        let (message_area_content, message_area_height) = match self.page.view {
                            View::Team => (
                                &mut self.team_message_area_content,
                                &mut self.team_message_area_height,
                            ),
                            View::Chat => (
                                &mut self.chat_message_area_content,
                                &mut self.chat_message_area_height,
                            ),
                            _ => return Task::none(), // Should never happen
                        };

                        // Handle sizing

                        let line_count = message_area_content.line_count();
                        let new_height = 33.0 + line_count as f32 * 21.0;

                        *message_area_height = if new_height > max_area_height {
                            max_area_height
                        } else {
                            new_height
                        };
                    }
                    Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        self.mouse_position = (position.x, position.y)
                    }
                    _ => {}
                }

                Task::none()
            }
            Message::ToggleShift(value) => {
                self.shift_held_down = value;
                Task::none()
            }
            Message::KeyPressed(key) => {
                match key {
                    Key::Named(Named::Escape) => {
                        self.expanded_image = None;
                    }
                    _ => {}
                }
                Task::none()
            }

            // UI interactions
            Message::MessageAreaEdit(action) => {
                let max_area_height = 0.5 * self.window_height;

                // Determine the current message area and height
                let (message_area_content, message_area_height) = match self.page.view {
                    View::Team => (
                        &mut self.team_message_area_content,
                        &mut self.team_message_area_height,
                    ),
                    View::Chat => (
                        &mut self.chat_message_area_content,
                        &mut self.chat_message_area_height,
                    ),
                    _ => return Task::none(), // Should never happen
                };

                let conversation_id = match self.page.view {
                    View::Team => self.page.current_channel_id.clone().unwrap(),
                    View::Chat => self.page.current_chat_id.clone().unwrap(),
                    _ => "".to_string(),
                };

                let message_area_text = message_area_content.text();
                let cursor_line_index = message_area_content.cursor_position().0;
                let current_line = message_area_text.lines().nth(cursor_line_index);
                match action {
                    Action::Edit(Edit::Enter) => {
                        if self.shift_held_down {
                            if let Some(current_line) = current_line {
                                if current_line.starts_with("- ") && current_line != "- " {
                                    message_area_content.perform(action);
                                    content_send(message_area_content, "- ");
                                } else if current_line == "- " {
                                    // Low-key a hotfix but it works
                                    message_area_content.perform(Action::SelectLine);
                                    message_area_content.perform(Action::Edit(Edit::Backspace));
                                } else if Regex::new(r"^\d+\. .+").unwrap().is_match(current_line) {
                                    message_area_content.perform(action);
                                    if let Some((number, _)) = current_line.split_once('.') {
                                        if let Ok(number) = number.parse::<u64>() {
                                            content_send(
                                                message_area_content,
                                                &format!("{}. ", number + 1),
                                            );
                                        }
                                    }
                                } else if Regex::new(r"^\d+\. $").unwrap().is_match(current_line) {
                                    message_area_content.perform(Action::SelectLine);
                                    message_area_content.perform(Action::Edit(Edit::Backspace));
                                } else {
                                    message_area_content.perform(action);
                                }
                            } else {
                                message_area_content.perform(action);
                            }
                        } else if message_area_content.text() != "\n".to_string() {
                            // Post a message instead if the content is not empty

                            let subject_text = if self.page.view == View::Team {
                                Some(self.subject_input_value.clone())
                            } else {
                                None
                            };

                            match self.page.view {
                                View::Team => self.team_message_area_content = Content::new(),
                                View::Chat => self.chat_message_area_content = Content::new(),
                                _ => {}
                            }

                            *message_area_height = 54.0;

                            self.subject_input_value = "".to_string();

                            let me_id = self.me.id.clone();

                            let me_display_name = self.me.display_name.clone();

                            let acess_tokens_arc = self.access_tokens.clone();
                            let tenant = self.tenant.clone();

                            if !conversation_id.starts_with("draft:") {
                                return post_message_task(
                                    message_area_text,
                                    acess_tokens_arc,
                                    tenant,
                                    conversation_id,
                                    me_id,
                                    me_display_name,
                                    subject_text,
                                );
                            } else {
                                let current_chat = self
                                    .chats
                                    .iter()
                                    .find(|chat| chat.id == conversation_id)
                                    .unwrap();

                                let members: Vec<ThreadMember> = current_chat
                                    .members
                                    .iter()
                                    .map(|memeber| ThreadMember {
                                        id: memeber.mri.clone(),
                                        role: "Admin".to_string(),
                                        share_history_time: None,
                                    })
                                    .collect();

                                let new_thread = Thread {
                                    members: members,
                                    properties: if current_chat.members.len() == 2 {
                                        // You and the other person (one on one chat)
                                        Some(ThreadProperties {
                                            thread_type: "chat".to_string(),
                                            fixed_roster: Some(true),
                                            unique_roster_thread: Some(true),
                                        })
                                    } else {
                                        Some(ThreadProperties {
                                            thread_type: "chat".to_string(),
                                            fixed_roster: Some(false),
                                            unique_roster_thread: Some(false),
                                        })
                                    },
                                };

                                let body = serde_json::to_string(&new_thread).unwrap();

                                return Task::perform(
                                    async move {
                                        let access_token = get_or_gen_token(
                                            acess_tokens_arc,
                                            "https://ic3.teams.office.com/.default".to_string(),
                                            &tenant,
                                        )
                                        .await;

                                        let thread_link =
                                            start_thread(&access_token, body).await.unwrap();

                                        (conversation_id, thread_link)
                                    },
                                    move |(conversation_id, thread_link)| {
                                        Message::CreatedGroupChat(
                                            conversation_id,
                                            thread_link,
                                            message_area_text.clone(),
                                        )
                                    },
                                );
                            }
                        }
                    }
                    _ => message_area_content.perform(action),
                }

                // Handle sizing

                let line_count = message_area_content.line_count();
                let new_height = 33.0 + line_count as f32 * 21.0;

                *message_area_height = if new_height > max_area_height {
                    max_area_height
                } else {
                    new_height
                };

                if self.page.view == View::Chat {
                    if self.should_send_typing && !conversation_id.starts_with("draft:") {
                        self.should_send_typing = false;

                        let acess_tokens_arc = self.access_tokens.clone();
                        let tenant = self.tenant.clone();
                        let conversation_id = self.page.current_chat_id.clone().unwrap();

                        return Task::batch(vec![
                            Task::perform(
                                async move {
                                    let access_token = get_or_gen_token(
                                        acess_tokens_arc,
                                        "https://ic3.teams.office.com/.default".to_string(),
                                        &tenant,
                                    )
                                    .await;

                                    let body = "{\"content\":\"\",\"contenttype\":\"Application/Message\",\"messagetype\":\"Control/Typing\"}";

                                    send_message(&access_token, conversation_id, body.to_string())
                                        .await
                                        .unwrap();
                                },
                                Message::DoNothing,
                            ),
                            Task::perform(
                                async {
                                    sleep(Duration::from_secs(4)).await;
                                },
                                Message::AllowPostIsTyping,
                            ),
                        ]);
                    }
                }
                Task::none()
            }
            Message::MessageAreaAction(action) => {
                let (content, message_area_height) = match self.page.view {
                    View::Team => (
                        &mut self.team_message_area_content,
                        &mut self.team_message_area_height,
                    ),
                    View::Chat => (
                        &mut self.chat_message_area_content,
                        &mut self.chat_message_area_height,
                    ),
                    _ => return Task::none(), // Should never happen
                };

                let selection = content.selection();

                match action {
                    MessageAreaAction::Bold => {
                        content_send(content, "**");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }

                        content_send(content, "**");
                    }
                    MessageAreaAction::Italic => {
                        content_send(content, "*");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }

                        content_send(content, "*");
                    }
                    MessageAreaAction::Underline => {
                        content_send(content, "<u>");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }

                        content_send(content, "</u>");
                    }
                    MessageAreaAction::Striketrough => {
                        content_send(content, "~~");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }

                        content_send(content, "~~");
                    }
                    MessageAreaAction::Blockquote => {
                        if content.text() != "\n" {
                            content_send(content, "\n\n");
                        }

                        content_send(content, "> ");
                    }
                    MessageAreaAction::Link => {
                        content_send(content, "[");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }
                        content_send(content, "](url)");
                    }
                    MessageAreaAction::Image => {
                        content_send(content, "![");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }
                        content_send(content, "](url)");
                    }
                    MessageAreaAction::Code => {
                        content_send(content, "`");

                        if let Some(selection) = selection {
                            for char in selection.chars() {
                                content.perform(Action::Edit(Edit::Insert(char)));
                            }
                        }
                        content_send(content, "`");
                    }
                    MessageAreaAction::List => {
                        if let Some(selection) = selection {
                            let mut lines = selection.lines().peekable();

                            while let Some(line) = lines.next() {
                                content_send(content, &format!("- {}", line));
                                if lines.peek().is_some() {
                                    content_send(content, "\n");
                                }
                            }
                        } else {
                            content_send(content, "- ");
                        }
                    }
                    MessageAreaAction::OrderedList => {
                        if let Some(selection) = selection {
                            let mut lines = selection.lines().enumerate().peekable();

                            while let Some((index, line)) = lines.next() {
                                content_send(content, &format!("{}. {}", index + 1, line));

                                if lines.peek().is_some() {
                                    content_send(content, "\n");
                                }
                            }
                        } else {
                            content_send(content, "1.  ");
                        }
                    }
                }

                // Handle sizing

                let max_area_height = 0.5 * self.window_height;

                let line_count = content.line_count();
                let new_height = 33.0 + line_count as f32 * 21.0;

                *message_area_height = if new_height > max_area_height {
                    max_area_height
                } else {
                    new_height
                };

                Task::none()
            }
            Message::AllowPostIsTyping(()) => {
                self.should_send_typing = true;
                Task::none()
            }
            Message::LinkClicked(url) => {
                if !webbrowser::open(url.as_str()).is_ok() {
                    eprintln!("Failed to open link : {}", url);
                }

                Task::none()
            }
            Message::OpenHome => {
                let page = Page {
                    view: View::Homepage,
                    chat_body: ChatBody::Messages,
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: self.page.current_chat_id.clone(),
                };
                self.page = page.clone();
                self.history.push(page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
            }
            Message::OpenTeam(team_id, channel_id) => {
                let team_page = Page {
                    view: View::Team,
                    chat_body: self.page.chat_body.clone(),
                    current_team_id: Some(team_id),
                    current_channel_id: Some(channel_id),
                    current_chat_id: self.page.current_chat_id.clone(),
                };

                self.page = team_page.clone();
                self.history.push(team_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
            }
            Message::OpenChat(thread_id) => {
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                let chat_page = Page {
                    view: View::Chat,
                    chat_body: ChatBody::Messages,
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: Some(thread_id.clone()),
                };

                self.page = chat_page.clone();
                self.history.push(chat_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                self.add_users_checked.clear();
                self.search_users_input_value = "".to_string();

                if !thread_id.starts_with("draft:") {
                    return Task::batch(vec![
                        snap_to(Id::new("conversation_column"), RelativeOffset::END),
                        Task::perform(
                            async move {
                                let time = get_epoch_ms();

                                let body = format!(
                                    "{{\"consumptionhorizon\":\"{};{};{}\"}}",
                                    time, time, time
                                );

                                let access_token = get_or_gen_token(
                                    access_tokens_arc,
                                    "https://ic3.teams.office.com/.default".to_string(),
                                    &tenant,
                                )
                                .await;

                                consumption_horizon(&access_token, thread_id.clone(), body)
                                    .await
                                    .unwrap();

                                thread_id
                            },
                            Message::ReadChat,
                        ),
                    ]);
                } else {
                    return snap_to(Id::new("conversation_column"), RelativeOffset::END);
                };
            }
            Message::OpenCurrentChat => {
                let chat_id = self.page.current_chat_id.clone();
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                let chat_page = Page {
                    view: View::Chat,
                    chat_body: ChatBody::Messages,
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: chat_id.clone(),
                };

                self.page = chat_page.clone();
                self.history.push(chat_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                self.add_users_checked.clear();

                if let Some(chat_id) = chat_id {
                    if !chat_id.starts_with("draft:") {
                        return Task::batch(vec![
                            snap_to(Id::new("conversation_column"), RelativeOffset::END),
                            Task::perform(
                                async move {
                                    let time = get_epoch_ms();

                                    let body = format!(
                                        "{{\"consumptionhorizon\":\"{};{};{}\"}}",
                                        time, time, time
                                    );

                                    let access_token = get_or_gen_token(
                                        access_tokens_arc,
                                        "https://ic3.teams.office.com/.default".to_string(),
                                        &tenant,
                                    )
                                    .await;

                                    consumption_horizon(&access_token, chat_id.clone(), body)
                                        .await
                                        .unwrap();

                                    chat_id
                                },
                                Message::ReadChat,
                            ),
                        ]);
                    } else {
                        return snap_to(Id::new("conversation_column"), RelativeOffset::END);
                    };
                }
                Task::none()
            }
            Message::ReadChat(thread_id) => {
                if let Some(chat) = self.chats.iter_mut().find(|chat| chat.id == thread_id) {
                    chat.is_read = Some(true);
                }
                Task::none()
            }
            Message::ToggleReplyOptions(conversation_id) => {
                let reply_options = &mut self.reply_options;
                let option = reply_options.entry(conversation_id).or_insert(false);
                *option = !*option;

                Task::none()
            }
            Message::ShowChatMessageOptions(chat_id) => {
                // Do not show the hover options if the emoji picker is open
                if self.emoji_picker_toggle.location.is_none() {
                    self.chat_message_options.insert(chat_id, true);
                }
                Task::none()
            }
            Message::StopShowChatMessageOptions(chat_id) => {
                // Do not hide the options for the current shown option if the emoji picker is open
                if self.emoji_picker_toggle.location.is_none() {
                    self.chat_message_options.insert(chat_id, false);
                } else {
                    // This ensures that the options menu is hidding after leaving the reaction menu
                    if self.emoji_picker_hide_options.is_none() {
                        self.emoji_picker_hide_options = Some(chat_id)
                    }
                }
                Task::none()
            }
            Message::HistoryBack => {
                if self.history_index != 0 {
                    self.history_index -= 1;
                    self.page = self.history[self.history_index].clone();
                }
                Task::none()
            }
            Message::HistoryForward => {
                if self.history_index != self.history.len() - 1 {
                    self.history_index += 1;
                    self.page = self.history[self.history_index].clone();
                }
                Task::none()
            }
            Message::ExpandImage(identifier, image_type) => {
                self.expanded_image = Some((identifier, image_type));
                Task::none()
            }
            Message::StopExpandImage => {
                self.expanded_image = None;
                Task::none()
            }
            Message::SearchTeamsContentChanged(content) => {
                self.search_teams_input_value = content;
                Task::none()
            }
            Message::SearchChatsContentChanged(content) => {
                self.search_chats_input_value = content;
                Task::none()
            }
            Message::SearchUsersContentChanged(content) => {
                self.search_users_input_value = content;
                Task::none()
            }
            Message::SubjectInputContentChanged(content) => {
                self.subject_input_value = content;
                Task::none()
            }
            Message::ToggleNewChatMenu => {
                self.add_users_checked.clear();
                self.search_users_input_value = "".to_string();

                if self.page.chat_body == ChatBody::Start {
                    self.page.chat_body = ChatBody::Messages;
                    Task::none()
                } else {
                    self.page.chat_body = ChatBody::Start;
                    focus(text_input::Id::new("search_users_input"))
                }
            }
            Message::Reply(message_content, display_name, message_id) => {
                let area_content = &mut self.chat_message_area_content;

                if area_content.text() != "\n" {
                    content_send(area_content, "\n\n");
                }

                content_send(
                    area_content,
                    &format!(
                        ">[{}][{}] {}\n",
                        display_name.unwrap_or("Unknown User".to_string()),
                        message_id.unwrap_or("0".to_string()),
                        if let Some(message_content) = message_content {
                            get_html_preview(&message_content)
                        } else {
                            "".to_string()
                        },
                    ),
                );

                let max_area_height = 0.5 * self.window_height;

                let line_count = area_content.line_count();
                let new_height = 33.0 + line_count as f32 * 21.0;

                self.chat_message_area_height = if new_height > max_area_height {
                    max_area_height
                } else {
                    new_height
                };

                Task::none()
            }

            // Teams requests
            Message::GotActivities(activities) => {
                let mut tasks = vec![];

                // If not read, fetch the conversation of the activity and add it to activity_expanded_conversations with GotExpandedActivity
                for activity_message in &activities {
                    if activity_message
                        .properties
                        .clone()
                        .unwrap()
                        .is_read
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    let message_activity_id = activity_message.id.clone().unwrap().to_string();

                    if let Some(activity) = activity_message.properties.clone().unwrap().activity {
                        let access_tokens_arc = self.access_tokens.clone();
                        let tenant = self.tenant.clone();
                        tasks.push(Task::perform(
                            async move {
                                let access_token = get_or_gen_token(
                                    access_tokens_arc,
                                    "https://ic3.teams.office.com/.default".to_string(),
                                    &tenant,
                                )
                                .await;

                                let conversation = conversations(
                                    &access_token,
                                    activity.source_thread_id.clone(),
                                    Some(
                                        activity
                                            .source_reply_chain_id
                                            .unwrap_or(activity.source_message_id),
                                    ),
                                )
                                .await
                                .unwrap();

                                (message_activity_id, conversation.messages)
                            },
                            |result| Message::GotExpandedActivity(result.0, result.1),
                        ));
                    }
                }

                self.activities = activities;
                Task::batch(tasks)
            }
            Message::GotUsers(user_profiles, profile) => {
                self.users = user_profiles;
                self.me = profile;
                Task::none()
            }
            Message::GotUserDetails(teams, chats) => {
                self.teams = teams;
                self.chats = chats;
                Task::none()
            }
            // UI initiated
            Message::ToggleExpandActivity(thread_id, message_id, message_activity_id) => {
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                if let Some(activity) = self
                    .activity_expanded_conversations
                    .get_mut(&message_activity_id)
                {
                    if activity.0 {
                        activity.0 = false;
                        return Task::perform(
                            async move {
                                let access_token = get_or_gen_token(
                                    access_tokens_arc,
                                    "https://ic3.teams.office.com/.default".to_string(),
                                    &tenant,
                                )
                                .await;
                                is_read(
                                    &access_token,
                                    "48:notifications".to_string(),
                                    message_activity_id.to_string(),
                                    "{\"isread\":\"true\"}".to_string(),
                                )
                                .await
                                .unwrap();
                            },
                            Message::DoNothing,
                        );
                    } else {
                        activity.0 = true;
                        return Task::none(); // Send isread false instead ?
                    }
                }

                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            access_tokens_arc,
                            "https://ic3.teams.office.com/.default".to_string(),
                            &tenant,
                        )
                        .await;

                        let conversation =
                            conversations(&access_token, thread_id.clone(), Some(message_id))
                                .await
                                .unwrap();

                        (message_activity_id, conversation.messages)
                    },
                    |result| Message::GotExpandedActivity(result.0, result.1),
                )
            }
            Message::GotExpandedActivity(message_activity_id, messages) => {
                self.activity_expanded_conversations
                    .insert(message_activity_id, (true, messages));
                Task::none()
            }
            Message::PrefetchChat(thread_id) => {
                let thread_id_clone = thread_id.clone();
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                if !thread_id.starts_with("draft:") {
                    Task::perform(
                        async move {
                            let access_token = get_or_gen_token(
                                access_tokens_arc,
                                "https://ic3.teams.office.com/.default".to_string(),
                                &tenant,
                            )
                            .await;

                            conversations(&access_token, thread_id, None).await.unwrap()
                        },
                        move |result| {
                            Message::GotChatConversations(thread_id_clone.clone(), result)
                        }, // This calls a message
                    )
                } else {
                    Task::none()
                }
            }
            Message::PrefetchCurrentChat => {
                if let Some(chat_id) = self.page.current_chat_id.clone() {
                    let chat_id_clone = chat_id.clone();
                    let access_tokens_arc = self.access_tokens.clone();
                    let tenant = self.tenant.clone();

                    if !chat_id.starts_with("draft:") {
                        return Task::perform(
                            async move {
                                let access_token = get_or_gen_token(
                                    access_tokens_arc,
                                    "https://ic3.teams.office.com/.default".to_string(),
                                    &tenant,
                                )
                                .await;

                                conversations(&access_token, chat_id_clone, None)
                                    .await
                                    .unwrap()
                            },
                            move |result| Message::GotChatConversations(chat_id.clone(), result), // This calls a message
                        );
                    }
                }
                Task::none()
            }
            Message::PrefetchTeam(team_id, channel_id) => {
                let channel_id_clone = channel_id.clone();
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc,
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                            &tenant,
                        )
                        .await;

                        team_conversations(&access_token, team_id, channel_id)
                            .await
                            .unwrap()
                    },
                    move |result| Message::GotConversations(channel_id_clone.clone(), result), // This calls a message
                )
            }
            Message::GotConversations(channel_id, conversations) => {
                self.team_conversations.insert(channel_id, conversations);
                Task::none()
            }
            Message::GotChatConversations(thread_id, conversations) => {
                self.chat_conversations
                    .insert(thread_id, conversations.messages);
                Task::none()
            }
            Message::OnScroll(viewport) => {
                let max_scroll = viewport.content_bounds().height - viewport.bounds().height;
                let scroll = max_scroll - viewport.absolute_offset().y;
                let percentage_scroll = 1.0 - (scroll / max_scroll);
                self.scrollbar_percentage_scroll = percentage_scroll;
                self.scrollbar_scroll = scroll as u64;
                Task::none()
            }
            Message::PostMessage => {
                let (message_area_content, message_area_height) = match self.page.view {
                    View::Team => (
                        &mut self.team_message_area_content,
                        &mut self.team_message_area_height,
                    ),
                    View::Chat => (
                        &mut self.chat_message_area_content,
                        &mut self.chat_message_area_height,
                    ),
                    _ => return Task::none(), // Should never happen
                };

                let subject_text = if self.page.view == View::Team {
                    Some(self.subject_input_value.clone())
                } else {
                    None
                };

                let message_area_text = message_area_content.text();

                match self.page.view {
                    View::Team => self.team_message_area_content = Content::new(),
                    View::Chat => self.chat_message_area_content = Content::new(),
                    _ => {}
                }

                *message_area_height = 54.0;

                self.subject_input_value = "".to_string();

                let conversation_id = match self.page.view {
                    View::Team => self.page.current_channel_id.clone().unwrap(),
                    View::Chat => self.page.current_chat_id.clone().unwrap(),
                    _ => "".to_string(),
                };

                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                let me_id = self.me.id.clone();
                let me_display_name = self.me.display_name.clone();

                if !conversation_id.starts_with("draft:") {
                    return post_message_task(
                        message_area_text,
                        acess_tokens_arc,
                        tenant,
                        conversation_id,
                        me_id,
                        me_display_name,
                        subject_text,
                    );
                } else {
                    let current_chat = self
                        .chats
                        .iter()
                        .find(|chat| chat.id == conversation_id)
                        .unwrap();

                    let members: Vec<ThreadMember> = current_chat
                        .members
                        .iter()
                        .map(|memeber| ThreadMember {
                            id: memeber.mri.clone(),
                            role: "Admin".to_string(),
                            share_history_time: None,
                        })
                        .collect();

                    let new_thread = Thread {
                        members: members,
                        properties: Some(ThreadProperties {
                            thread_type: "chat".to_string(),
                            fixed_roster: Some(false),
                            unique_roster_thread: Some(false),
                        }),
                    };

                    let body = serde_json::to_string(&new_thread).unwrap();

                    return Task::perform(
                        async move {
                            let access_token = get_or_gen_token(
                                acess_tokens_arc,
                                "https://ic3.teams.office.com/.default".to_string(),
                                &tenant,
                            )
                            .await;

                            let thread_link = start_thread(&access_token, body).await.unwrap();

                            (conversation_id, thread_link)
                        },
                        move |(conversation_id, thread_link)| {
                            Message::CreatedGroupChat(
                                conversation_id,
                                thread_link,
                                message_area_text.clone(),
                            )
                        },
                    );
                }
            }
            Message::StartChat(user_ids) => {
                let mut user_ids = user_ids;
                // Check if the one to one chat already exists
                if user_ids.len() == 1 {
                    for chat in &self.chats {
                        if chat.members.len() == 2 {
                            if chat
                                .members
                                .iter()
                                .any(|member| member.mri.replace("8:orgid:", "") == user_ids[0])
                            {
                                let chat_id = chat.id.clone();

                                self.page.chat_body = ChatBody::Messages;
                                self.page.current_chat_id = Some(chat_id.clone());

                                let chat_id_clone = chat_id.clone();
                                let access_tokens_arc = self.access_tokens.clone();
                                let tenant = self.tenant.clone();

                                // Prefetch the chat in case it wasn't already fetched
                                return Task::batch(vec![
                                    Task::perform(
                                        async move {
                                            let access_token = get_or_gen_token(
                                                access_tokens_arc,
                                                "https://ic3.teams.office.com/.default".to_string(),
                                                &tenant,
                                            )
                                            .await;

                                            conversations(&access_token, chat_id, None)
                                                .await
                                                .unwrap()
                                        },
                                        move |result| {
                                            Message::GotChatConversations(
                                                chat_id_clone.clone(),
                                                result,
                                            )
                                        },
                                    ),
                                    snap_to(Id::new("conversation_column"), RelativeOffset::END),
                                ]);
                            }
                        }
                    }
                }

                user_ids.push(self.me.id.clone());

                let members: Vec<ChatMember> = user_ids
                    .iter()
                    .map(|user_id| ChatMember {
                        mri: format!("8:orgid:{}", user_id), // Users should all be of 8:orgid:
                        is_muted: None,
                        object_id: None,
                        is_identity_masked: None,
                        role: None,
                    })
                    .collect();

                let mut rng = StdRng::from_os_rng();

                let draft_id = format!("draft:{}", rng.random::<u64>()); // generate the temproary draft chat_id randomly

                self.chats.insert(
                    0,
                    Chat {
                        id: draft_id.clone(), // Signal that it isn't a real chat
                        members: members,
                        is_read: None,
                        is_high_importance: None,
                        is_one_on_one: if user_ids.len() > 2 {
                            Some(false)
                        } else {
                            Some(true)
                        },
                        is_conversation_deleted: None,
                        is_external: None,
                        is_messaging_disabled: None,
                        is_disabled: None,
                        title: None,
                        last_message: None,
                        is_last_message_from_me: None,
                        chat_sub_type: None,
                        last_join_at: None,
                        created_at: None,
                        creator: None,
                        hidden: None,
                        added_by: None,
                        chat_type: Some("draft".to_string()),
                        picture: None,
                    },
                );

                self.page.chat_body = ChatBody::Messages;

                self.page.current_chat_id = Some(draft_id);

                Task::none()
            }
            Message::CreatedGroupChat(draft_id, thread_link, message) => {
                let chat_id =
                    thread_link.replace("https://emea.ng.msg.teams.microsoft.com/v1/threads/", "");

                self.chats
                    .iter_mut()
                    .find(|chat| chat.id == draft_id)
                    .unwrap()
                    .chat_type = Some("chat".to_string());

                self.chats
                    .iter_mut()
                    .find(|chat| chat.id == draft_id)
                    .unwrap()
                    .id = chat_id.clone();

                self.chat_conversations.insert(chat_id.clone(), vec![]);

                self.page.current_chat_id = Some(chat_id.clone());

                post_message_task(
                    message,
                    self.access_tokens.clone(),
                    self.tenant.clone(),
                    chat_id,
                    self.me.id.clone(),
                    self.me.display_name.clone(),
                    None,
                )
                .chain(snap_to(Id::new("conversation_column"), RelativeOffset::END))
            }
            Message::AddToGroupChat(chat_id, user_ids) => {
                // Handle if it is a draft
                if chat_id.starts_with("draft:") {
                    self.chats
                        .iter_mut()
                        .find(|chat| chat.id == chat_id)
                        .unwrap()
                        .members
                        .extend(user_ids.iter().map(|user_id| ChatMember {
                            mri: format!("8:orgid:{}", user_id), // Users should all be of 8:orgid:
                            role: None,
                            is_muted: None,
                            object_id: None,
                            is_identity_masked: None,
                        }));

                    self.page.chat_body = ChatBody::Messages;

                    return Task::none();
                }

                let members: Vec<ThreadMember> = user_ids
                    .iter()
                    .map(|user_id| ThreadMember {
                        id: format!("8:orgid:{}", user_id), // Users should all be of 8:orgid:
                        role: "Admin".to_string(),
                        share_history_time: Some(-1),
                    })
                    .collect();

                let chat_thread = Thread {
                    members: members,
                    properties: None,
                };

                // Serialize to JSON
                let body = serde_json::to_string_pretty(&chat_thread).unwrap();

                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc,
                            "https://ic3.teams.office.com/.default".to_string(),
                            &tenant,
                        )
                        .await;

                        add_member(&access_token, chat_id.clone(), body)
                            .await
                            .unwrap();

                        (chat_id, user_ids)
                    },
                    |(chat_id, user_ids)| Message::AddedToGroupChat(chat_id, user_ids),
                )
            }
            Message::AddedToGroupChat(chat_id, user_ids) => {
                let mut chat = self.chats.iter_mut().find(|chat| chat.id == chat_id);
                // Shouldn't happen
                if let Some(chat) = chat.as_mut() {
                    for user_id in user_ids {
                        chat.members.push(ChatMember {
                            mri: user_id,
                            is_muted: None,
                            object_id: None,
                            role: None,
                            is_identity_masked: None,
                        });
                    }
                }

                self.page.chat_body = ChatBody::Messages;
                Task::none()
            }
            Message::FetchTeamImage(identifier, picture_e_tag, group_id, display_name) => {
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc,
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                            &tenant,
                        )
                        .await;

                        let picture_e_tag = picture_e_tag;

                        let bytes = authorize_team_picture(
                            &access_token,
                            group_id,
                            picture_e_tag.clone(),
                            display_name,
                        )
                        .await
                        .unwrap();

                        save_cached_image(identifier, "jpeg", bytes);
                    },
                    Message::DoNothing,
                )
            }
            Message::FetchUserImage(identifier, user_id, display_name) => {
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc,
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                            &tenant,
                        )
                        .await;

                        let user_id = user_id;

                        let bytes =
                            authorize_profile_picture(&access_token, user_id.clone(), display_name)
                                .await
                                .unwrap();

                        save_cached_image(identifier, "jpeg", bytes);
                    },
                    Message::DoNothing,
                )
            }
            Message::FetchMergedProfilePicture(identifier, users) => {
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                let user_id = self.me.id.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc,
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                            &tenant,
                        )
                        .await;

                        let bytes = authorize_merged_profile_picture(&access_token, users, user_id)
                            .await
                            .unwrap();

                        save_cached_image(identifier, "jpeg", bytes);
                    },
                    Message::DoNothing,
                )
            }
            Message::AuthorizeImage(url, identifier) => {
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc.clone(),
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                            &tenant,
                        )
                        .await;

                        let skype_token =
                            get_or_gen_skype_token(acess_tokens_arc, access_token).await;

                        let bytes = authorize_image(&skype_token, url.clone()).await.unwrap();

                        save_cached_image(identifier, "jpeg", bytes);
                    },
                    Message::DoNothing,
                )
            }
            Message::DownloadImage(url, identifier) => Task::perform(
                async move {
                    let client = Client::new();
                    let response = client.get(url).send().await.unwrap();
                    let bytes = response.bytes().await.unwrap();

                    save_cached_image(identifier, "gif", bytes);
                },
                Message::DoNothing,
            ),
            Message::DownloadFile(file) => {
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                if let Some(share_url) = file.file_info.share_url {
                    let share_id = BASE64_URL_SAFE.encode(share_url);

                    return Task::perform(
                        async move {
                            let access_token = get_or_gen_token(
                                acess_tokens_arc.clone(),
                                "https://graph.microsoft.com/.default".to_string(),
                                &tenant,
                            )
                            .await;

                            let url = format!(
                                "https://graph.microsoft.com/v1.0/shares/u!{}/driveItem/content",
                                share_id
                            );

                            sharepoint_download_file(&access_token, url).await.unwrap()
                        },
                        Message::DownloadedFile,
                    );
                } else if let Some(site_url) = file.file_info.site_url {
                    let web_url = site_url.split("/").nth(2).unwrap().to_string();
                    let site_path =
                        site_url.replace(format!("https://{}/sites/", web_url).as_str(), "");
                    let item_id = file.item_id.unwrap().to_string();

                    return Task::perform(
                        async move {
                            let access_token = get_or_gen_token(
                                acess_tokens_arc.clone(),
                                "https://graph.microsoft.com/.default".to_string(),
                                &tenant,
                            )
                            .await;

                            let site_id = site_info(&access_token, web_url, site_path)
                                .await
                                .unwrap()
                                .id;

                            let url = format!(
                                "https://graph.microsoft.com/v1.0/sites/{}/drive/items/{}/content",
                                site_id, item_id
                            );

                            sharepoint_download_file(&access_token, url).await.unwrap()
                        },
                        Message::DownloadedFile,
                    );
                }
                Task::none()
            }
            Message::DownloadedFile(url) => {
                if !webbrowser::open(url.as_str()).is_ok() {
                    eprintln!("Failed to open link : {}", url);
                }
                Task::none()
            }
            Message::ToggleShowChatMembers => {
                if self.page.chat_body != ChatBody::Members {
                    self.page.chat_body = ChatBody::Members;
                    snap_to(Id::new("members_column"), RelativeOffset { x: 0.0, y: 0.0 })
                } else {
                    self.page.chat_body = ChatBody::Messages;
                    snap_to(
                        Id::new("conversation_column"),
                        RelativeOffset {
                            x: 0.0,
                            y: self.scrollbar_percentage_scroll,
                        },
                    )
                }
            }
            Message::ToggleShowChatAdd => {
                if self.page.chat_body != ChatBody::Add {
                    self.page.chat_body = ChatBody::Add;
                    Task::batch(vec![
                        focus(text_input::Id::new("search_users_input")),
                        snap_to(Id::new("members_column"), RelativeOffset { x: 0.0, y: 0.0 }),
                    ])
                } else {
                    self.page.chat_body = ChatBody::Messages;
                    snap_to(
                        Id::new("conversation_column"),
                        RelativeOffset {
                            x: 0.0,
                            y: self.scrollbar_percentage_scroll,
                        },
                    )
                }
            }
            Message::ToggleUserCheckbox(checked, id) => {
                self.add_users_checked.insert(id, !checked);
                Task::none()
            }
            Message::ToggleEmojiPicker(location, action) => {
                if self.emoji_picker_toggle.action == EmojiPickerAction::None {
                    self.emoji_picker_toggle = EmojiPickerInfo {
                        action: action,
                        location: location,
                    };
                } else {
                    self.emoji_picker_toggle = EmojiPickerInfo {
                        action: EmojiPickerAction::None,
                        location: location,
                    };

                    // In case the emoji picker was opened from a message options tab. Yes, it's a bit of a hotfix
                    if let Some(chat_id) = &self.emoji_picker_hide_options {
                        self.chat_message_options.insert(chat_id.clone(), false);
                        self.emoji_picker_hide_options = None
                    }
                }
                self.last_mouse_position = self.mouse_position;
                Task::none()
            }
            Message::EmojiPickerPicked(emoji_id, emoji_unicode) => {
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                match &self.emoji_picker_toggle.action {
                    EmojiPickerAction::Send => {
                        let content = match self.page.view {
                            View::Team => &mut self.team_message_area_content,
                            View::Chat => &mut self.chat_message_area_content,
                            _ => return Task::none(),
                        };

                        content_send(content, &emoji_unicode);

                        Task::none()
                    }
                    EmojiPickerAction::Reaction(message_id) => {
                        let time = get_epoch_ms();

                        let body = format!(
                            "{{\"emotions\":{{\"key\":\"{}\",\"value\":{}}}}}",
                            emoji_id, time
                        );

                        let thread_id = match self.page.view {
                            View::Team => self.page.current_team_id.clone().unwrap(),
                            View::Chat => self.page.current_chat_id.clone().unwrap(),
                            _ => return Task::none(),
                        };

                        let message_id = message_id.clone();

                        Task::perform(
                            async move {
                                let access_token = get_or_gen_token(
                                    access_tokens_arc,
                                    "https://ic3.teams.office.com/.default".to_string(),
                                    &tenant,
                                )
                                .await;

                                emotions(&access_token, &thread_id, &message_id, body)
                                    .await
                                    .unwrap();
                            },
                            Message::DoNothing,
                        )
                    }
                    _ => return Task::none(),
                }
            }

            // Websockets
            Message::WSConnected(info) => {
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                // This will be more useful in the future...
                self.websockets_connection_info = Some(info.clone());

                let surl = info.surl.clone();
                let endpoint = info.endpoint.clone();

                let mut chat_users = HashSet::new();

                for chat in &self.chats {
                    for member in &chat.members {
                        chat_users.insert(member.mri.clone());
                    }
                }

                let subscriptions = chat_users
                    .iter()
                    .map(|mri| format!(r#"{{"mri":"{}","source":"ups"}}"#, mri))
                    .collect::<Vec<_>>()
                    .join(",");

                let body = format!(
                    r#"{{
                           "trouterUri":"{}/unifiedPresenceService",
                           "shouldPurgePreviousSubscriptions":false,
                           "subscriptionsToAdd":[{}],
                           "subscriptionsToRemove":[]
                       }}"#,
                    surl, subscriptions
                );

                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            access_tokens_arc,
                            "https://presence.teams.microsoft.com/.default".to_string(),
                            &tenant,
                        )
                        .await;

                        websockets_subscription(&access_token, &endpoint, &surl, body)
                            .await
                            .unwrap();
                    },
                    Message::DoNothing,
                )
            }
            Message::GotWSMessage(message) => {
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                let message = message.resource;
                if let Some(message_type) = &message.message_type {
                    if message_type == "Control/Typing" {
                        let chat_id = message.conversation_link.unwrap().replace(
                            "https://notifications.skype.net/v1/users/ME/conversations/",
                            "",
                        );
                        let user_id = message.from.unwrap();

                        let timeout_entry = self
                            .users_typing_timeouts
                            .entry(chat_id.clone())
                            .or_insert_with(HashMap::new);

                        if let Some(timeout) = timeout_entry.get(&user_id) {
                            timeout.abort();
                        } else {
                            //let entry = self.users_typing.entry(chat_id).or_insert_with(Vec::new);
                            //entry.push(user_id);
                        }

                        let user_id_clone = user_id.clone();

                        let (task, handle) = Task::perform(
                            async {
                                sleep(Duration::from_secs(4)).await;
                            },
                            move |_| {
                                Message::TypingTimeoutFinished(
                                    chat_id.clone(),
                                    user_id_clone.clone(),
                                )
                            },
                        )
                        .abortable();
                        timeout_entry.insert(user_id, handle);
                        return task;

                        //handle.abort();
                    } else {
                        match self.page.view {
                            View::Chat => {
                                let chat_id = message.conversation_link.clone().unwrap().replace(
                                    "https://notifications.skype.net/v1/users/ME/conversations/",
                                    "",
                                );

                                // Update conversations

                                if let Some(conversation) =
                                    self.chat_conversations.get_mut(&chat_id)
                                {
                                    if let Some(pos) =
                                        conversation.iter().position(|item| item.id == message.id)
                                    {
                                        conversation[pos] = message.clone();
                                    } else {
                                        conversation.insert(0, message.clone());
                                    }
                                }

                                if let Some(current_chat_id) = &self.page.current_chat_id {
                                    if let Some(pos) =
                                        self.chats.iter_mut().position(|chat| chat.id == chat_id)
                                    {
                                        let mut chat = self.chats.remove(pos);

                                        // Add notification

                                        if &chat_id != current_chat_id {
                                            chat.is_read = Some(false);
                                        }

                                        // Move the chat to the top

                                        self.chats.insert(0, chat);
                                    }
                                }

                                // Clear  is typing timeouts

                                let user_id = message.from.unwrap();

                                if let Some(timeoutes) =
                                    self.users_typing_timeouts.get_mut(&chat_id)
                                {
                                    timeoutes.remove(&user_id);
                                }

                                // Tasks

                                let mut tasks = vec![];

                                if let Some(current_chat_id) = &self.page.current_chat_id {
                                    if &chat_id == current_chat_id {
                                        if self.scrollbar_scroll < 60 {
                                            tasks.push(snap_to(
                                                Id::new("conversation_column"),
                                                RelativeOffset::END,
                                            ))
                                        }
                                        tasks.push(Task::perform(
                                            async move {
                                                let time = get_epoch_ms();

                                                let body = format!(
                                                    "{{\"consumptionhorizon\":\"{};{};{}\"}}",
                                                    time, time, time
                                                );

                                                let access_token = get_or_gen_token(
                                                    access_tokens_arc,
                                                    "https://ic3.teams.office.com/.default"
                                                        .to_string(),
                                                    &tenant,
                                                )
                                                .await;

                                                consumption_horizon(&access_token, chat_id, body)
                                                    .await
                                                    .unwrap();
                                            },
                                            Message::DoNothing,
                                        ))
                                    }
                                }

                                return Task::batch(tasks);
                            }
                            View::Team => {
                                let message_link_data = message
                                    .conversation_link
                                    .clone()
                                    .unwrap()
                                    .replace(
                                    "https://notifications.skype.net/v1/users/ME/conversations/",
                                    "",
                                );

                                let message_link_parts: Vec<&str> =
                                    message_link_data.split(";").collect();

                                let channel_id = *message_link_parts.get(0).unwrap();

                                if !channel_id.contains("@thread.tacv")
                                    || channel_id.contains("48:threads")
                                {
                                    // If not a "team channel" conversation return.
                                    return Task::none();
                                }

                                if message_link_parts.get(1).is_none() {
                                    // Should't happpen, but just to be sure
                                    return Task::none();
                                }

                                let message_link_id =
                                    message_link_parts.get(1).unwrap().replace("messageid=", "");

                                // Check if it is a new post, post edit, new reply, reply edit

                                let mut reply_chain_exists = false;
                                let mut reply_chain_message_exists = false;

                                if let Some(conversation) =
                                    self.team_conversations.get_mut(channel_id)
                                {
                                    for conversation in &mut conversation.reply_chains {
                                        if message_link_id == conversation.id {
                                            reply_chain_exists = true;

                                            for (pos, conversation_message) in
                                                conversation.messages.iter_mut().enumerate()
                                            {
                                                if conversation_message.id == message.id {
                                                    // Edit
                                                    conversation.messages[pos] = message.clone();
                                                    reply_chain_message_exists = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    if !reply_chain_exists {
                                        // New post
                                        conversation.reply_chains.insert(
                                            0,
                                            Conversation {
                                                messages: vec![message.clone()],
                                                container_id: message.id.clone().unwrap(),
                                                id: message.id.clone().unwrap(),
                                                latest_delivery_time: message
                                                    .original_arrival_time
                                                    .unwrap_or("n/a".to_string()),
                                            },
                                        );
                                    } else if !reply_chain_message_exists {
                                        // New reply
                                        for conversation in &mut conversation.reply_chains {
                                            if conversation.id == message_link_id {
                                                conversation.messages.insert(0, message.clone());
                                            }
                                        }
                                    }
                                }

                                if self.scrollbar_scroll < 60 {
                                    return snap_to(
                                        Id::new("conversation_column"),
                                        RelativeOffset::END,
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }

                Task::none()
            }
            Message::GotWSPresences(presences) => {
                for presence in presences.presence {
                    self.user_presences.insert(presence.mri.clone(), presence);
                }
                Task::none()
            }
            Message::TypingTimeoutFinished(chat_id, user_id) => {
                self.users_typing_timeouts
                    .get_mut(&chat_id)
                    .unwrap()
                    .remove(&user_id);
                Task::none()
            }

            // Other
            Message::DoNothing(_) => Task::none(),
            Message::Join => {
                println!("Join message called!");
                Task::none()
            }
            Message::Hello(message) => {
                println!("Hello {message}");
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            event::listen().map(Message::EventOccurred),
            keyboard::on_key_press(|key, _modifiers| Some(Message::KeyPressed(key))),
            keyboard::on_key_press(|key, _modifiers| match key {
                Key::Named(Named::Shift) => Some(Message::ToggleShift(true)),
                _ => None,
            }),
            keyboard::on_key_release(|key, _modifiers| match key {
                Key::Named(Named::Shift) => Some(Message::ToggleShift(false)),
                _ => None,
            }),
        ];

        if self.is_authorized {
            subscriptions.push(
                Subscription::run_with_id(
                    "websockets",
                    connect(self.access_tokens.clone(), self.tenant.clone()),
                )
                .map(|response_type| match response_type {
                    WebsocketResponse::Connected(info) => Message::WSConnected(info),
                    WebsocketResponse::Message(value) => Message::GotWSMessage(value),
                    WebsocketResponse::Presences(value) => Message::GotWSPresences(value),
                    WebsocketResponse::Other(value) => Message::DoNothing(()),
                }),
            )
        }

        Subscription::batch(subscriptions)
    }

    // The default theming system is not used, except for the background
    fn theme(&self) -> Theme {
        let custom_palette = iced::theme::palette::Palette {
            background: self.theme.colors.background,
            text: Color::new(1.0, 0.0, 0.0, 1.0),
            primary: Color::new(1.0, 0.0, 0.0, 1.0),
            success: Color::new(1.0, 0.0, 0.0, 1.0),
            danger: Color::new(1.0, 0.0, 0.0, 1.0),
        };

        Theme::custom("Squads Dark".to_string(), custom_palette)
    }
}

pub fn main() -> iced::Result {
    iced::application("Squads", Counter::update, Counter::view)
        .window_size(Size {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        })
        .subscription(Counter::subscription)
        .theme(Counter::theme)
        .font(include_bytes!("../resources/OpenSans-Regular-COLR.ttf").as_slice()) // Increases startup time with about 100 ms...
        .default_font(Font::with_name("Open Sans Twemoji"))
        .run_with(Counter::new)
}
