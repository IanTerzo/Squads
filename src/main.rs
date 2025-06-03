mod api;
mod api_types;
mod components;
mod parsing;
use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use iced::task::Handle;
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
    event, keyboard, window, Color, Element, Event, Point, Size, Subscription, Task, Theme,
};
use pages::app;
use pages::page_chat::chat;
use pages::page_home::home;
use pages::page_login::login;
use pages::page_team::team;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs};
use style::global_theme;
use tokio::time::sleep;
use types::*;
use utils::{get_cache, get_epoch_ms, save_to_cache};
use webbrowser;
use websockets::{
    connect, websockets_subscription, ConnectionInfo, Presence, Presences, WebsocketMessage,
    WebsocketResponse,
};

const WINDOW_WIDTH: f32 = 1240.0;
const WINDOW_HEIGHT: f32 = 780.0;

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
    shift_held_down: bool,
    scrollbar_scroll: u64,
    scrollbar_percentage_scroll: f32,
    should_send_typing: bool,
    users_typing_timeouts: HashMap<String, HashMap<String, Handle>>, // Where string is the chat id and the other string is the user id

    // UI state
    reply_options: HashMap<String, bool>, // String is the conversation id
    chat_message_options: HashMap<String, bool>, // String is the message id
    team_conversations: HashMap<String, TeamConversations>, // String is the team id
    chat_conversations: HashMap<String, Vec<api::Message>>, // String is the thread id
    activity_expanded_conversations: HashMap<String, Vec<api::Message>>, // String is the thread id
    search_teams_input_value: String,
    search_chats_input_value: String,
    team_message_area_content: Content,
    team_message_area_height: f32,
    chat_message_area_content: Content,
    chat_message_area_height: f32,
    expanded_image: Option<(String, String)>,
    show_members: bool,

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
    AllowPostIsTyping(()),
    // Teams requests
    GotActivities(Vec<api::Message>),
    GotUsers(HashMap<String, Profile>, Profile),
    GotUserDetails(Vec<Team>, Vec<Chat>),
    // UI initiated
    ExpandActivity(String, u64, String),
    GotExpandedActivity(String, Vec<api::Message>), //callback
    PrefetchChat(String),
    PrefetchCurrentChat,
    GotChatConversations(String, Conversations), //callback
    PrefetchTeam(String, String),
    GotConversations(String, TeamConversations), //callback
    OnScroll(Viewport),
    PostMessage,
    FetchTeamImage(String, String, String, String),
    FetchUserImage(String, String, String),
    FetchMergedProfilePicture(String, Vec<(String, String)>),
    AuthorizeImage(String, String),
    DownloadImage(String, String),
    DownloadFile(File),
    DownloadedFile(String),
    ToggleShowChatMembers,

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

                let users = users(&access_token_graph);

                let mut profile_map = HashMap::new();

                for profile in users.await.unwrap().value {
                    profile_map.insert(profile.id.clone(), profile);
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
    me_display_name: String,
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
                imdisplayname: &me_display_name,
                clientmessageid: &message_id.to_string(),
                call_id: "",
                state: 0,
                version: "0",
                amsreferences: vec![],
                properties: Properties {
                    importance: "",
                    subject: None,
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
        let file_content = fs::read_to_string("resources/emojis.json").unwrap();
        let emojies: HashMap<String, String> = serde_json::from_str(&file_content).unwrap();

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
            show_members: false,
            history: vec![Page {
                view: View::Homepage,
                current_team_id: None,
                current_channel_id: None,
                current_chat_id: first_chat,
            }],
            websockets_connection_info: None,
            user_presences: HashMap::new(),
            expanded_image: None,
            should_send_typing: true,
            history_index: 0,
            emoji_map: emojies,
            search_teams_input_value: "".to_string(),
            search_chats_input_value: "".to_string(),
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
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
                        &self.chats,
                        &conversation,
                        &self.chat_message_options,
                        &self.emoji_map,
                        &self.users,
                        &self.user_presences,
                        &self.me,
                        self.search_chats_input_value.clone(),
                        &self.chat_message_area_content,
                        &self.chat_message_area_height,
                        &self.show_members,
                    ),
                    if let Some(expanded_image) = self.expanded_image.clone() {
                        Some(c_expanded_image(expanded_image.0, expanded_image.1))
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

                let message_area_text = message_area_content.text();

                match action {
                    Action::Edit(Edit::Enter) => {
                        if self.shift_held_down {
                            message_area_content.perform(action);
                        } else {
                            // Post a message instead

                            match self.page.view {
                                View::Team => self.team_message_area_content = Content::new(),
                                View::Chat => self.chat_message_area_content = Content::new(),
                                _ => {}
                            }

                            let conversation_id = match self.page.view {
                                View::Team => self.page.current_channel_id.clone().unwrap(),
                                View::Chat => self.page.current_chat_id.clone().unwrap(),
                                _ => "".to_string(),
                            };

                            let me_id = self.me.id.clone();

                            let me_display_name = self.me.display_name.clone().unwrap();

                            let acess_tokens_arc = self.access_tokens.clone();
                            let tenant = self.tenant.clone();
                            return post_message_task(
                                message_area_text,
                                acess_tokens_arc,
                                tenant,
                                conversation_id,
                                me_id,
                                me_display_name,
                            );
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
                    if self.should_send_typing {
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
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: self.page.current_chat_id.clone(),
                };
                self.page = page.clone();
                self.history.push(page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);
                self.show_members = false;

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
            }
            Message::OpenTeam(team_id, channel_id) => {
                let team_page = Page {
                    view: View::Team,
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
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: Some(thread_id.clone()),
                };

                self.page = chat_page.clone();
                self.history.push(chat_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);
                self.show_members = false;

                Task::batch(vec![
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
                ])
            }
            Message::OpenCurrentChat => {
                let chat_id = self.page.current_chat_id.clone();
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();

                let chat_page = Page {
                    view: View::Chat,
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: chat_id.clone(),
                };

                self.page = chat_page.clone();
                self.history.push(chat_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                if let Some(chat_id) = chat_id {
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
                self.chat_message_options.insert(chat_id, true);
                Task::none()
            }
            Message::StopShowChatMessageOptions(chat_id) => {
                self.chat_message_options.insert(chat_id, false);
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

                    if let Some(activity) = activity_message
                        .properties
                        .clone()
                        .unwrap()
                        .activity
                    {
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
            Message::ExpandActivity(thread_id, message_id, message_activity_id) => {
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
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
                    .insert(message_activity_id, messages);
                Task::none()
            }
            Message::PrefetchChat(thread_id) => {
                let thread_id_clone = thread_id.clone();
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
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
                    move |result| Message::GotChatConversations(thread_id_clone.clone(), result), // This calls a message
                )
            }
            Message::PrefetchCurrentChat => {
                if let Some(chat_id) = self.page.current_chat_id.clone() {
                    let chat_id_clone = chat_id.clone();
                    let access_tokens_arc = self.access_tokens.clone();
                    let tenant = self.tenant.clone();
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

                let message_area_text = message_area_content.text();

                match self.page.view {
                    View::Team => self.team_message_area_content = Content::new(),
                    View::Chat => self.chat_message_area_content = Content::new(),
                    _ => {}
                }

                *message_area_height = 54.0;

                let conversation_id = match self.page.view {
                    View::Team => self.page.current_channel_id.clone().unwrap(),
                    View::Chat => self.page.current_chat_id.clone().unwrap(),
                    _ => "".to_string(),
                };

                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                let me_id = self.me.id.clone();
                let me_display_name = self.me.display_name.clone().unwrap();
                post_message_task(
                    message_area_text,
                    acess_tokens_arc,
                    tenant,
                    conversation_id,
                    me_id,
                    me_display_name,
                )
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
                self.show_members = !self.show_members;
                if self.show_members {
                    snap_to(Id::new("members_column"), RelativeOffset { x: 0.0, y: 0.0 })
                } else {
                    snap_to(
                        Id::new("conversation_column"),
                        RelativeOffset {
                            x: 0.0,
                            y: self.scrollbar_percentage_scroll,
                        },
                    )
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
                        let time = get_epoch_ms();

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
                //println!("{message:#?}");
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
        .font(include_bytes!("../resources/Twemoji-15.1.0.ttf").as_slice()) // Increases startup time with about 100 ms...
        .run_with(Counter::new)
}
