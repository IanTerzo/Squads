mod api;
mod api_types;
mod components;
mod parsing;
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
    authorize_team_picture, conversations, gen_refresh_token_from_device_code, me, send_message,
    team_conversations, teams_me, users, AccessToken, Chat, Conversations, DeviceCodeInfo, Profile,
    Team, TeamConversations,
};
use auth::{get_or_gen_skype_token, get_or_gen_token};
use components::cached_image::save_cached_image;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::widget::scrollable::{snap_to, Id, RelativeOffset};
use iced::widget::text_editor::{self, Action, Content, Edit};
use iced::{event, keyboard, window, Color, Element, Event, Size, Subscription, Task, Theme};
use pages::app;
use pages::page_chat::chat;
use pages::page_home::home;
use pages::page_login::login;
use pages::page_team::team;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs};
use style::global_theme;
use types::*;
use utils::{get_cache, save_to_cache};
use webbrowser;

const WINDOW_WIDTH: f32 = 1240.0;
const WINDOW_HEIGHT: f32 = 780.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum View {
    Login,
    Homepage,
    Team,
    Chat,
}

// Any information needed to display the current page
#[derive(Debug, Clone)]
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

    // UI state
    reply_options: HashMap<String, bool>, // String is the conversation id
    chat_message_options: HashMap<String, bool>, // String is the message id
    team_conversations: HashMap<String, TeamConversations>, // String is the team id
    chat_conversations: HashMap<String, Vec<api::Message>>, // String is the thread id
    activity_expanded_conversations: HashMap<String, Vec<api::Message>>, // String is the thread id
    search_teams_input_value: String,
    team_message_area_content: Content,
    team_message_area_height: f32,
    chat_message_area_content: Content,
    chat_message_area_height: f32,

    // Teams requested data
    me: Profile,
    users: HashMap<String, Profile>,
    teams: Vec<Team>,
    chats: Vec<Chat>,
    activities: Vec<api::Message>,
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

    // UI interactions
    MessageAreaEdit(text_editor::Action),
    LinkClicked(String),
    Jump(Page),
    OpenTeam(String, String),
    OpenChat(String),
    ToggleReplyOptions(String),
    ShowChatMessageOptions(String),
    StopShowChatMessageOptions(String),
    HistoryBack,
    HistoryForward,
    ContentChanged(String),

    // Teams requests
    GotActivities(Vec<api::Message>),
    GotUsers(HashMap<String, Profile>, Profile),
    GotUserDetails(Vec<Team>, Vec<Chat>),
    // UI initiated
    ExpandActivity(String, u64, String),
    GotExpandedActivity(String, Vec<api::Message>), //callback
    PrefetchChat(String),
    GotChatConversations(String, Result<Conversations, String>), //callback
    PrefetchTeam(String, String),
    GotConversations(String, Result<TeamConversations, String>), //callback
    PostMessage,
    FetchTeamImage(String, String, String, String),
    FetchUserImage(String, String, String),
    FetchMergedProfilePicture(String, Vec<(String, String)>),
    AuthorizeImage(String, String),

    // Other
    DoNothing(()),
    Join, // For testing
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
                );

                let activity_messages =
                    conversations(&access_token_ic3, "48:notifications".to_string(), None).unwrap();

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
                );

                let user_details = teams_me(&access_token_chatsvcagg).unwrap();

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
                );

                let users = users(&access_token_graph);

                let mut profile_map = HashMap::new();

                for profile in users.unwrap().value {
                    profile_map.insert(profile.id.clone(), profile);
                }

                let profile = me(&access_token_graph).unwrap();

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
            );

            let mut rng = rand::rng();
            let message_id: u64 = rng.random(); // generate the message_id randomly

            let message = TeamsMessage {
                id: "-1".to_string(),
                msg_type: "Message".to_string(),
                conversationid: conversation_id.clone(),
                conversation_link: format!("blah/{}", conversation_id),
                from: format!("8:orgid:{}", me_id),
                composetime: "2025-03-06T11:04:18.265Z".to_string(),
                originalarrivaltime: "2025-03-06T11:04:18.265Z".to_string(),
                content: html,
                messagetype: "RichText/Html".to_string(),
                contenttype: "Text".to_string(),
                imdisplayname: me_display_name,
                clientmessageid: message_id.to_string(),
                call_id: "".to_string(),
                state: 0,
                version: "0".to_string(),
                amsreferences: vec![],
                properties: Properties {
                    importance: "".to_string(),
                    subject: "SUBJECT".to_string(),
                    title: "".to_string(),
                    cards: "[]".to_string(),
                    links: "[]".to_string(),
                    mentions: "[]".to_string(),
                    onbehalfof: None,
                    files: "[]".to_string(),
                    policy_violation: None,
                    format_variant: "TEAMS".to_string(),
                },
                post_type: "Standard".to_string(),
                cross_post_channels: vec![],
            };

            // Convert the struct into a JSON string
            let body = serde_json::to_string_pretty(&message).unwrap();

            send_message(&access_token, conversation_id, body).unwrap();

            println!("Posted!");
        },
        Message::DoNothing,
    )
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

        let tenant = "organizations".to_string(); // Why does this work?

        let counter_self = Self {
            page: Page {
                view: if has_refresh_token {
                    View::Homepage
                } else {
                    View::Login
                },
                current_team_id: None,
                current_channel_id: None,
                current_chat_id: None,
            },
            theme: global_theme(),
            device_user_code: None,
            device_code: "".to_string(),
            tenant: tenant.clone(),
            team_message_area_content: Content::new(),
            team_message_area_height: 54.0,
            chat_message_area_content: Content::new(),
            chat_message_area_height: 54.0,
            reply_options: HashMap::new(),
            chat_message_options: HashMap::new(),
            history: vec![Page {
                view: View::Homepage,
                current_team_id: None,
                current_channel_id: None,
                current_chat_id: None,
            }],
            history_index: 0,
            emoji_map: emojies,
            search_teams_input_value: "".to_string(),
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
                    async move { api::gen_device_code(tenant).unwrap() },
                    Message::GotDeviceCodeInfo,
                )
            },
        )
    }

    fn view(&self) -> Element<Message> {
        //println!("view called");

        match self.page.view {
            View::Login => login(&self.theme, &self.device_user_code),
            View::Homepage => {
                let search_value = self.search_teams_input_value.clone();

                app(
                    &self.theme,
                    home(
                        &self.theme,
                        &self.teams,
                        &self.activities,
                        self.activity_expanded_conversations.clone(),
                        &self.emoji_map,
                        &self.users,
                        self.window_width,
                        search_value,
                    ),
                )
            }
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
                )
            }
            View::Chat => {
                let conversation =
                    if let Some(current_channel_id) = self.page.current_chat_id.as_ref() {
                        self.chat_conversations.get(current_channel_id)
                    } else {
                        None
                    };

                app(
                    &self.theme,
                    chat(
                        &self.theme,
                        &self.chats,
                        &conversation,
                        &self.chat_message_options,
                        &self.emoji_map,
                        &self.users.to_owned(),
                        self.me.to_owned().id,
                        &self.chat_message_area_content,
                        &self.chat_message_area_height,
                    ),
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
                        );
                        if let Ok(access_token) = result {
                            refresh_token = Some(access_token);
                            println!("Code polling succeeded.")
                        } else {
                            println!("Code polling failed: {:#?}", result.err())
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
                        let result = gen_refresh_token_from_device_code(device_code, tenant);
                        if let Ok(access_token) = result {
                            refresh_token = Some(access_token);
                            println!("Code polling succeeded.")
                        } else {
                            println!("Code polling failed: {:#?}", result.err())
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
                    }
                    _ => {}
                }

                Task::none()
            }
            Message::ToggleShift(value) => {
                self.shift_held_down = value;
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

                Task::none()
            }
            Message::LinkClicked(url) => {
                if !webbrowser::open(url.as_str()).is_ok() {
                    eprintln!("Failed to open link : {}", url);
                }

                Task::none()
            }
            Message::Jump(page) => {
                self.page = page;
                Task::none()
            }
            Message::OpenTeam(team_id, channel_id) => {
                let team_page = Page {
                    view: View::Team,
                    current_team_id: Some(team_id),
                    current_channel_id: Some(channel_id),
                    current_chat_id: None,
                };

                self.page = team_page.clone();
                self.history.push(team_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
            }
            Message::OpenChat(thread_id) => {
                let chat_page = Page {
                    view: View::Chat,
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: Some(thread_id),
                };

                self.page = chat_page.clone();
                self.history.push(chat_page);
                self.history_index += 1;
                self.history.truncate(self.history_index + 1);

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
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
            Message::ContentChanged(content) => {
                println!("{}", content);
                self.search_teams_input_value = content;
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

                    let activity = activity_message
                        .properties
                        .clone()
                        .unwrap()
                        .activity
                        .unwrap();

                    tasks.push(Task::perform(
                        {
                            let access_tokens_arc = self.access_tokens.clone();
                            let tenant = self.tenant.clone();
                            async move {
                                let access_token = get_or_gen_token(
                                    access_tokens_arc,
                                    "https://ic3.teams.office.com/.default".to_string(),
                                    &tenant,
                                );

                                let conversation = conversations(
                                    &access_token,
                                    activity.source_thread_id.clone(),
                                    Some(
                                        activity
                                            .source_reply_chain_id
                                            .unwrap_or(activity.source_message_id),
                                    ),
                                )
                                .unwrap();

                                (message_activity_id, conversation.messages)
                            }
                        },
                        |result| Message::GotExpandedActivity(result.0, result.1),
                    ));
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
                        );
                        let conversation =
                            conversations(&access_token, thread_id.clone(), Some(message_id))
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
                let channel_id_clone = thread_id.clone();
                let access_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            access_tokens_arc,
                            "https://ic3.teams.office.com/.default".to_string(),
                            &tenant,
                        );
                        conversations(&access_token, thread_id, None)
                    },
                    move |result| Message::GotChatConversations(channel_id_clone.clone(), result), // This calls a message
                )
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
                        );
                        team_conversations(&access_token, team_id, channel_id)
                    },
                    move |result| Message::GotConversations(channel_id_clone.clone(), result), // This calls a message
                )
            }
            Message::GotConversations(channel_id, conversations) => {
                self.team_conversations
                    .insert(channel_id, conversations.unwrap());
                Task::none()
            }
            Message::GotChatConversations(thread_id, conversations) => {
                self.chat_conversations
                    .insert(thread_id, conversations.unwrap().messages);
                Task::none()
            }
            Message::PostMessage => {
                let message_area_text = match self.page.view {
                    View::Team => self.team_message_area_content.text(),
                    View::Chat => self.chat_message_area_content.text(),
                    _ => "".to_string(),
                };

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
                        );

                        let picture_e_tag = picture_e_tag;

                        let bytes = authorize_team_picture(
                            &access_token,
                            group_id,
                            picture_e_tag.clone(),
                            display_name,
                        )
                        .unwrap();

                        save_cached_image(identifier, bytes);
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
                        );
                        let user_id = user_id;

                        let bytes =
                            authorize_profile_picture(&access_token, user_id.clone(), display_name)
                                .unwrap();

                        save_cached_image(identifier, bytes);
                    },
                    Message::DoNothing,
                )
            }

            Message::FetchMergedProfilePicture(identifier, users) => {
                let acess_tokens_arc = self.access_tokens.clone();
                let tenant = self.tenant.clone();
                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            acess_tokens_arc,
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                            &tenant,
                        );
                        let bytes = authorize_merged_profile_picture(&access_token, users).unwrap();

                        save_cached_image(identifier, bytes);
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
                        );

                        let skype_token = get_or_gen_skype_token(acess_tokens_arc, access_token);
                        let url = url;

                        let bytes = authorize_image(&skype_token, url.clone()).unwrap();

                        save_cached_image(identifier, bytes);
                    },
                    Message::DoNothing,
                )
            }

            // Other
            Message::DoNothing(_) => Task::none(),
            Message::Join => {
                println!("Join message called!");
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            event::listen().map(Message::EventOccurred),
            keyboard::on_key_press(|key, _modifiers| match key {
                Key::Named(Named::Shift) => Some(Message::ToggleShift(true)),
                _ => None,
            }),
            keyboard::on_key_release(|key, _modifiers| match key {
                Key::Named(Named::Shift) => Some(Message::ToggleShift(false)),
                _ => None,
            }),
        ])
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
