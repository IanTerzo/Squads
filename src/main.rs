use iced::widget::scrollable::{snap_to, Id, RelativeOffset};
use iced::widget::text_editor::{self, Content};
use iced::{event, window, Color, Element, Event, Size, Subscription, Task, Theme};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, fs, io::Write};
use webbrowser;
mod components;
use components::cached_image::save_cached_image;
use directories::ProjectDirs;
mod api;
mod parsing;
use parsing::parse_message_markdown;
mod style;
use style::global_theme;
mod utils;
mod widgets;
use api::{
    authorize_image, authorize_merged_profile_picture, authorize_profile_picture,
    authorize_team_picture, conversations, me, send_message, team_conversations, teams_me, users,
    AccessToken, Chat, Conversations, Profile, Team, TeamConversations,
};
mod auth;
use auth::{get_or_gen_skype_token, get_or_gen_token};
mod pages;
use pages::app;
use pages::page_chat::chat;
use pages::page_home::home;
use pages::page_login::login;
use pages::page_team::team;

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
    page: Page,
    theme: style::Theme,
    reply_options: HashMap<String, bool>, // String is the conversation id
    chat_message_options: HashMap<String, bool>, // String is the message id
    history: Vec<Page>,
    emoji_map: HashMap<String, String>,
    search_teams_input_value: String,
    team_message_area_content: Content,
    team_message_area_height: f32,
    chat_message_area_content: Content,
    chat_message_area_height: f32,
    window_width: f32,
    window_height: f32,
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    me: Profile,
    users: HashMap<String, Profile>,
    teams: Vec<Team>,
    chats: Vec<Chat>,
    team_conversations: HashMap<String, TeamConversations>, // String is the team id
    chat_conversations: HashMap<String, Vec<api::Message>>, // String is the thread id
    activity_expanded_conversations: Arc<RwLock<HashMap<String, Vec<api::Message>>>>, // String is the thread id
    activities: Vec<api::Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MessageAreaEdit(text_editor::Action, String),
    EventOccurred(Event),
    Authorized(()),
    DoNothing(()),
    LinkClicked(String),
    Join,
    Jump(Page),
    PostMessage,
    ExpandActivity(String, u64, String),
    OpenChat(String),
    PrefetchChat(String),
    GotChatConversations(String, Result<Conversations, String>),
    ToggleReplyOptions(String),
    ShowChatMessageOptions(String),
    StopShowChatMessageOptions(String),
    HistoryBack,
    OpenTeam(String, String),
    PrefetchTeam(String, String),
    GotConversations(String, Result<TeamConversations, String>),
    GotActivities(Vec<api::Message>),
    GotUsers(HashMap<String, Profile>, Profile),
    GotUserDetails(Vec<Team>, Vec<Chat>),
    FetchTeamImage(String, String, String, String),
    FetchUserImage(String, String, String),
    FetchMergedProfilePicture(String, Vec<(String, String)>),
    AuthorizeImage(String, String),
    ContentChanged(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TeamsMessage {
    id: String,
    #[serde(rename = "type")]
    msg_type: String,
    conversationid: String,
    conversation_link: String,
    from: String,
    composetime: String,
    originalarrivaltime: String,
    content: String,
    messagetype: String,
    contenttype: String,
    imdisplayname: String,
    clientmessageid: String,
    call_id: String,
    state: i32,
    version: String,
    amsreferences: Vec<String>,
    properties: Properties,
    post_type: String,
    cross_post_channels: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Properties {
    importance: String,
    subject: String,
    title: String,
    cards: String,
    links: String,
    mentions: String,
    onbehalfof: Option<String>,
    files: String,
    policy_violation: Option<String>,
    format_variant: String,
}

fn save_to_cache<T>(filename: &str, content: &T)
where
    T: Serialize,
{
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    cache_dir.push(filename);

    let json = serde_json::to_string_pretty(content).expect("Failed to serialize content");
    let mut file = fs::File::create(cache_dir).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

fn get_cache<T: DeserializeOwned>(filename: &str) -> Option<T> {
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    cache_dir.push(filename);

    if cache_dir.exists() {
        let file_content = fs::read_to_string(cache_dir).ok()?;
        serde_json::from_str(&file_content).ok()
    } else {
        None
    }
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        let file_content = fs::read_to_string("resources/emojis.json").unwrap();
        let emojies: HashMap<String, String> = serde_json::from_str(&file_content).unwrap();

        let access_tokens = Arc::new(RwLock::new(HashMap::new()));

        let arc_me = Arc::new(RwLock::new(Profile::default()));

        if let Some(cached) = get_cache::<HashMap<String, AccessToken>>("access_tokens.json") {
            *access_tokens.write().unwrap() = cached;
        }

        let teams = get_cache::<Vec<Team>>("teams.json").unwrap_or(Vec::new());

        let chats = get_cache::<Vec<Chat>>("chats.json").unwrap_or(Vec::new());

        let user_profiles =
            get_cache::<HashMap<String, Profile>>("users.json").unwrap_or(HashMap::new());

        let profile = get_cache::<Profile>("me.json").unwrap_or(Profile::default());

        let mut counter_self = Self {
            page: Page {
                view: View::Homepage,
                current_team_id: None,
                current_channel_id: None,
                current_chat_id: None,
            },
            theme: global_theme(),
            team_message_area_content: Content::new(),
            team_message_area_height: 54.0,
            chat_message_area_content: Content::new(),
            chat_message_area_height: 54.0,
            reply_options: HashMap::new(),
            chat_message_options: HashMap::new(),
            history: Vec::new(),
            emoji_map: emojies,
            search_teams_input_value: "".to_string(),
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            access_tokens: access_tokens.clone(),
            users: user_profiles,
            me: profile,
            teams: teams.clone(),
            chats: chats.clone(),
            activity_expanded_conversations: Arc::new(RwLock::new(HashMap::new())),
            team_conversations: HashMap::new(),
            chat_conversations: HashMap::new(),
            activities: Vec::new(),
        };

        counter_self.history.push(counter_self.page.clone());

        // hotfix...
        let access_tokens_clone = access_tokens.clone();
        let access_tokens_clone2 = access_tokens.clone();
        let access_tokens_clone3 = access_tokens.clone();
        (
            counter_self,
            Task::batch(vec![
                Task::perform(
                    async move {
                        let access_token_ic3 = get_or_gen_token(
                            access_tokens,
                            "https://ic3.teams.office.com/.default".to_string(),
                        );

                        let activity_messages =
                            conversations(access_token_ic3, "48:notifications".to_string(), None)
                                .unwrap();

                        activity_messages.messages
                    },
                    Message::GotActivities,
                ),
                Task::perform(
                    async move {
                        let access_token_chatsvcagg = get_or_gen_token(
                            access_tokens_clone,
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                        );

                        let user_details = teams_me(access_token_chatsvcagg).unwrap();

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
                            access_tokens_clone2,
                            "https://graph.microsoft.com/.default".to_string(),
                        );

                        let users = users(access_token_graph.clone());

                        let mut profile_map = HashMap::new();

                        for profile in users.unwrap().value {
                            profile_map.insert(profile.id.clone(), profile);
                        }

                        let profile = me(access_token_graph).unwrap();

                        save_to_cache("users.json", &profile_map.to_owned());
                        save_to_cache("me.json", &profile.to_owned());

                        (profile_map, profile)
                    },
                    |result| Message::GotUsers(result.0, result.1),
                ),
            ])
            .chain(Task::perform(
                async move {
                    save_to_cache("access_tokens.json", &access_tokens_clone3.to_owned());
                },
                Message::Authorized,
            )),
        )
    }

    fn view(&self) -> Element<Message> {
        //println!("view called");

        match self.page.view {
            View::Login => app(&self.theme, login()),
            View::Homepage => {
                let expanded_conversations = match self.activity_expanded_conversations.try_read() {
                    Ok(guard) => guard.clone(),
                    Err(_) => HashMap::new(),
                };

                let search_value = self.search_teams_input_value.clone();

                app(
                    &self.theme,
                    home(
                        &self.theme,
                        &self.teams,
                        &self.activities,
                        expanded_conversations,
                        &self.emoji_map,
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
            Message::GotUserDetails(teams, chats) => {
                self.teams = teams;
                self.chats = chats;
                Task::none()
            }
            Message::GotActivities(activities) => {
                self.activities = activities;
                Task::none()
            }
            Message::GotUsers(user_profiles, profile) => {
                self.users = user_profiles;
                self.me = profile;
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

            Message::ExpandActivity(thread_id, message_id, message_activity_id) => Task::perform(
                {
                    let access_token = get_or_gen_token(
                        self.access_tokens.clone(),
                        "https://ic3.teams.office.com/.default".to_string(),
                    );
                    let arc_expanded_conversations = self.activity_expanded_conversations.clone();
                    async move {
                        let conversation =
                            conversations(access_token, thread_id.clone(), Some(message_id))
                                .unwrap();
                        arc_expanded_conversations
                            .write()
                            .unwrap()
                            .insert(message_activity_id, conversation.messages.clone());
                        println!("{conversation:#?}");
                    }
                },
                Message::DoNothing,
            ),
            Message::OpenChat(thread_id) => {
                let team_page = Page {
                    view: View::Chat,
                    current_team_id: None,
                    current_channel_id: None,
                    current_chat_id: Some(thread_id),
                };

                self.page = team_page.clone();
                self.history.push(team_page);

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
            }

            Message::PrefetchChat(thread_id) => {
                let channel_id_clone = thread_id.clone();
                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            self.access_tokens.clone(),
                            "https://ic3.teams.office.com/.default".to_string(),
                        );
                        async move { conversations(access_token, thread_id, None) }
                    },
                    move |result| Message::GotChatConversations(channel_id_clone.clone(), result), // This calls a message
                )
            }

            Message::GotChatConversations(thread_id, conversations) => {
                self.chat_conversations
                    .insert(thread_id, conversations.unwrap().messages);
                Task::none()
            }

            Message::MessageAreaEdit(action, area_id) => {
                let max_area_height = 0.5 * self.window_height;

                if area_id == "team" {
                    self.team_message_area_content.perform(action);
                    let line_count = self.team_message_area_content.line_count();
                    let new_height = 33.0 + line_count as f32 * 21.0;

                    if new_height > max_area_height {
                        self.team_message_area_height = max_area_height;
                    } else {
                        self.team_message_area_height = new_height;
                    }
                } else if area_id == "chat" {
                    self.chat_message_area_content.perform(action);
                    let line_count = self.chat_message_area_content.line_count();
                    let new_height = 33.0 + line_count as f32 * 21.0;

                    if new_height > max_area_height {
                        self.team_message_area_height = max_area_height;
                    } else {
                        self.chat_message_area_height = new_height;
                    }
                }
                Task::none()
            }
            Message::DoNothing(_) => Task::none(),

            Message::PostMessage => {
                let message_area_text = match self.page.view {
                    View::Team => self.team_message_area_content.text(),
                    View::Chat => self.chat_message_area_content.text(),
                    _ => "".to_string(),
                };

                let conversation_id = match self.page.view {
                    View::Team => self.page.current_channel_id.clone().unwrap(),
                    View::Chat => self.page.current_chat_id.clone().unwrap(),
                    _ => "".to_string(),
                };

                let html = parse_message_markdown(message_area_text);

                let access_token = get_or_gen_token(
                    self.access_tokens.clone(),
                    "https://ic3.teams.office.com/.default".to_string(),
                );

                let mut rng = rand::rng();
                let message_id: u64 = rng.random(); // generate the message_id randomly

                let message = TeamsMessage {
                    id: "-1".to_string(),
                    msg_type: "Message".to_string(),
                    conversationid: conversation_id.clone(),
                    conversation_link: format!("blah/{}", conversation_id),
                    from: format!("8:orgid:{}", self.me.id.clone()),
                    composetime: "2025-03-06T11:04:18.265Z".to_string(),
                    originalarrivaltime: "2025-03-06T11:04:18.265Z".to_string(),
                    content: html,
                    messagetype: "RichText/Html".to_string(),
                    contenttype: "Text".to_string(),
                    imdisplayname: self.me.display_name.clone().unwrap(),
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

                send_message(access_token, conversation_id, body).unwrap();
                println!("Posted!");

                Task::none()
            }
            Message::Authorized(_response) => {
                self.history.push(self.page.clone());
                Task::none()
            }

            Message::Join => {
                println!("Join message called!");
                Task::none()
            }
            Message::Jump(page) => {
                self.page = page;
                Task::none()
            }
            Message::LinkClicked(url) => {
                if !webbrowser::open(url.as_str()).is_ok() {
                    eprintln!("Failed to open link : {}", url);
                }

                Task::none()
            }
            Message::ToggleReplyOptions(conversation_id) => {
                let reply_options = &mut self.reply_options;
                let option = reply_options.entry(conversation_id).or_insert(false);
                *option = !*option;

                Task::none()
            }

            Message::HistoryBack => {
                self.page = self.history[0].clone(); // WILL FIX SOON!
                Task::none()
            }

            Message::FetchTeamImage(identifier, picture_e_tag, group_id, display_name) => {
                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            self.access_tokens.clone(),
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                        );
                        async {
                            let picture_e_tag = picture_e_tag;

                            let bytes = authorize_team_picture(
                                access_token,
                                group_id,
                                picture_e_tag.clone(),
                                display_name,
                            )
                            .unwrap();

                            save_cached_image(identifier, bytes);
                        }
                    },
                    Message::DoNothing,
                )
            }

            Message::FetchUserImage(identifier, user_id, display_name) => Task::perform(
                {
                    let access_token = get_or_gen_token(
                        self.access_tokens.clone(),
                        "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                    );
                    async {
                        let user_id = user_id;

                        let bytes =
                            authorize_profile_picture(access_token, user_id.clone(), display_name)
                                .unwrap();

                        save_cached_image(identifier, bytes);
                    }
                },
                Message::DoNothing,
            ),

            Message::FetchMergedProfilePicture(identifier, users) => Task::perform(
                {
                    let access_token = get_or_gen_token(
                        self.access_tokens.clone(),
                        "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                    );
                    async {
                        let bytes = authorize_merged_profile_picture(access_token, users).unwrap();

                        save_cached_image(identifier, bytes);
                    }
                },
                Message::DoNothing,
            ),

            Message::AuthorizeImage(url, identifier) => Task::perform(
                {
                    let access_token = get_or_gen_token(
                        self.access_tokens.clone(),
                        "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                    );

                    let skype_token =
                        get_or_gen_skype_token(self.access_tokens.clone(), access_token);

                    async {
                        let url = url;

                        let bytes = authorize_image(skype_token, url.clone()).unwrap();

                        save_cached_image(identifier, bytes);
                    }
                },
                Message::DoNothing,
            ),

            Message::OpenTeam(team_id, channel_id) => {
                let team_page = Page {
                    view: View::Team,
                    current_team_id: Some(team_id),
                    current_channel_id: Some(channel_id),
                    current_chat_id: None,
                };

                self.page = team_page.clone();
                self.history.push(team_page);

                snap_to(Id::new("conversation_column"), RelativeOffset::END)
            }

            Message::PrefetchTeam(team_id, channel_id) => {
                let channel_id_clone = channel_id.clone();
                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            self.access_tokens.clone(),
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                        );
                        async move { team_conversations(access_token, team_id, channel_id) }
                    },
                    move |result| Message::GotConversations(channel_id_clone.clone(), result), // This calls a message
                )
            }

            Message::GotConversations(channel_id, conversations) => {
                self.team_conversations
                    .insert(channel_id, conversations.unwrap());
                Task::none()
            }
            Message::ContentChanged(content) => {
                self.search_teams_input_value = content;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
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
