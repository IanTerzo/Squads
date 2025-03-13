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
    activity, authorize_image, authorize_profile_picture, authorize_team_picture, me, send_message,
    team_conversations, teams_me, users, AccessToken, Chat, Profile, Team, TeamConversations,
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
    current_team_id: String,
    current_channel_id: String,
    show_conversations: bool,
}

#[derive(Debug)]
struct Counter {
    page: Page,
    theme: style::Theme,
    reply_options: HashMap<String, bool>, // String is the conversation id
    history: Vec<Page>,
    emoji_map: HashMap<String, String>,
    search_teams_input_value: String,
    message_area_content: Content,
    message_area_height: f32,
    window_width: f32,
    window_height: f32,
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    me: Arc<RwLock<Profile>>,
    users: Arc<RwLock<HashMap<String, Profile>>>,
    teams: Arc<RwLock<Vec<Team>>>,
    teams_cached: Vec<Team>,
    chats: Arc<RwLock<Vec<Chat>>>,
    team_conversations: HashMap<String, TeamConversations>, // String is the team id
    activities: Arc<RwLock<Vec<api::Message>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    EventOccurred(Event),
    Authorized(()),
    DoNothing(()),
    LinkClicked(String),
    Join,
    Jump(Page),
    PostMessage,
    ToggleReplyOptions(String),
    HistoryBack,
    OpenTeam(String, String),
    FetchTeamImage(String, String, String, String),
    FetchUserImage(String, String, String),
    AuthorizeImage(String, String),
    ShowConversations(()),
    GotConversations(String, Result<TeamConversations, String>),
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
        let teams = Arc::new(RwLock::new(Vec::new()));
        let chats = Arc::new(RwLock::new(Vec::new()));
        let arc_users = Arc::new(RwLock::new(HashMap::new()));
        let arc_me = Arc::new(RwLock::new(Profile::default()));

        if let Some(cached) = get_cache::<HashMap<String, AccessToken>>("access_tokens.json") {
            *access_tokens.write().unwrap() = cached;
        }

        let mut teams_cached = Vec::new();
        if let Some(cached) = get_cache::<Vec<Team>>("teams.json") {
            teams_cached = cached.clone();
            *teams.write().unwrap() = cached;
        }
        if let Some(cached) = get_cache::<Vec<Chat>>("chats.json") {
            *chats.write().unwrap() = cached;
        }
        if let Some(cached) = get_cache::<HashMap<String, Profile>>("users.json") {
            *arc_users.write().unwrap() = cached;
        }
        if let Some(cached) = get_cache::<Profile>("me.json") {
            *arc_me.write().unwrap() = cached;
        }

        let arc_activities = Arc::new(RwLock::new(Vec::new()));

        let mut counter_self = Self {
            page: Page {
                view: View::Login,
                current_team_id: "0".to_string(),
                current_channel_id: "0".to_string(),
                show_conversations: false,
            },
            theme: global_theme(),
            message_area_height: 54.0,
            message_area_content: Content::new(),
            reply_options: HashMap::new(),
            history: Vec::new(),
            emoji_map: emojies,
            search_teams_input_value: "".to_string(),
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            access_tokens: access_tokens.clone(),
            me: arc_me.clone(),
            users: arc_users.clone(),
            teams: teams.clone(),
            teams_cached: teams_cached,
            chats: chats.clone(),
            team_conversations: HashMap::new(),
            activities: arc_activities.clone(),
        };

        //if cache.refresh_token.expires < get_epoch_s() {} show login page

        counter_self.page.view = View::Homepage;
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

                        let activity_messages = activity(access_token_ic3).unwrap();
                        let mut activities = arc_activities.write().unwrap();

                        *activities = activity_messages.messages;
                    },
                    Message::DoNothing,
                ),
                Task::perform(
                    async move {
                        let access_token_chatsvcagg = get_or_gen_token(
                            access_tokens_clone,
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                        );

                        let user_details = teams_me(access_token_chatsvcagg.clone()).unwrap();
                        let mut teams = teams.write().unwrap();
                        *teams = user_details.clone().teams;

                        let mut chats = chats.write().unwrap();
                        *chats = user_details.clone().chats;

                        save_to_cache("teams.json", &teams.to_owned());
                        save_to_cache("chats.json", &chats.to_owned());
                    },
                    Message::DoNothing,
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
                            profile_map.insert(profile.clone().id, profile);
                        }

                        let mut users = arc_users.write().unwrap();
                        *users = profile_map;

                        let mut profile = arc_me.write().unwrap();
                        *profile = me(access_token_graph).unwrap();

                        save_to_cache("users.json", &users.to_owned());
                        save_to_cache("me.json", &profile.to_owned());
                    },
                    Message::DoNothing,
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
                let teams = match self.teams.try_read() {
                    Ok(guard) => guard.clone(),
                    Err(_) => self.teams_cached.to_owned(), // A bit of a hotfix, but it works
                };

                let activities = match self.activities.try_read() {
                    Ok(guard) => guard.clone(),
                    Err(_) => Vec::new(),
                };

                let search_value = self.search_teams_input_value.clone();

                app(
                    &self.theme,
                    home(
                        &self.theme,
                        teams,
                        activities,
                        &self.emoji_map,
                        self.window_width,
                        search_value,
                    ),
                )
            }
            View::Team => {
                let current_team = self
                    .teams
                    .read()
                    .unwrap()
                    .iter()
                    .find(|team| team.id == self.page.current_team_id)
                    .unwrap()
                    .clone();

                let current_channel = current_team
                    .channels
                    .iter()
                    .find(|channel| channel.id == self.page.current_channel_id)
                    .unwrap()
                    .clone();

                let reply_options = self.reply_options.clone();

                // NOTE: We need to open the team page withou any conversations first, and then load the conversations, otherwise the app would feel unresposive if it froze until the conversations where loaded (and rendered into iced components)
                // That's why this exists. Better solutions are welcome.

                if self.page.show_conversations {
                    let conversation = self.team_conversations.get(&self.page.current_team_id);
                    app(
                        &self.theme,
                        team(
                            &self.theme,
                            current_team,
                            current_channel,
                            conversation.cloned(),
                            reply_options,
                            &self.emoji_map,
                            &self.message_area_content,
                            self.message_area_height,
                        ),
                    )
                } else {
                    app(
                        &self.theme,
                        team(
                            &self.theme,
                            current_team,
                            current_channel,
                            None,
                            reply_options,
                            &self.emoji_map,
                            &self.message_area_content,
                            self.message_area_height,
                        ),
                    )
                }
            }
            View::Chat => app(
                &self.theme,
                chat(
                    &self.theme,
                    self.chats.read().unwrap().to_owned(),
                    self.users.read().unwrap().to_owned(),
                    self.me.read().unwrap().to_owned().id,
                ),
            ),
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

                        if self.message_area_height > max_area_height {
                            self.message_area_height = max_area_height;
                        }
                    }
                    _ => {}
                }

                Task::none()
            }
            Message::Edit(action) => {
                let max_area_height = 0.5 * self.window_height;
                self.message_area_content.perform(action);
                let line_count = self.message_area_content.line_count();
                let new_height = 33.0 + line_count as f32 * 21.0;

                if new_height > max_area_height {
                    self.message_area_height = max_area_height;
                } else {
                    self.message_area_height = new_height;
                }

                Task::none()
            }
            Message::DoNothing(_) => Task::none(),

            Message::PostMessage => {
                let message_area_text = self.message_area_content.text();
                let html = parse_message_markdown(message_area_text);

                let access_token = get_or_gen_token(
                    self.access_tokens.clone(),
                    "https://ic3.teams.office.com/.default".to_string(),
                );

                let mut rng = rand::rng();
                let message_id: u64 = rng.random(); // generate the mssage_id randomly

                let message = TeamsMessage {
                    id: "-1".to_string(),
                    msg_type: "Message".to_string(),
                    conversationid: self.page.current_channel_id.clone(),
                    conversation_link: format!("blah/{}", self.page.current_channel_id.clone()),
                    from: format!("8:orgid:{}", self.me.read().unwrap().id.clone()),
                    composetime: "2025-03-06T11:04:18.265Z".to_string(),
                    originalarrivaltime: "2025-03-06T11:04:18.265Z".to_string(),
                    content: html,
                    messagetype: "RichText/Html".to_string(),
                    contenttype: "Text".to_string(),
                    imdisplayname: self.me.read().unwrap().display_name.clone().unwrap(),
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

                send_message(access_token, self.page.current_channel_id.clone(), body).unwrap();
                println!("Posted!");

                Task::none()
            }
            Message::Authorized(_response) => {
                self.page.view = View::Homepage;
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
                    current_team_id: team_id.clone(),
                    current_channel_id: channel_id.clone(),
                    show_conversations: false,
                };
                self.page = team_page.clone();

                Task::perform(async {}, Message::ShowConversations)
            }

            Message::ShowConversations(_) => {
                let team_page = Page {
                    view: View::Team,
                    current_team_id: self.page.current_team_id.clone(),
                    current_channel_id: self.page.current_channel_id.clone(),
                    show_conversations: true,
                };
                self.page = team_page.clone();
                self.history.push(team_page);

                let team_id = self.page.current_team_id.clone();
                let channel_id = self.page.current_channel_id.clone();
                let team_id_clone = self.page.current_team_id.clone();

                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            self.access_tokens.clone(),
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                        );
                        async move { team_conversations(access_token, team_id, channel_id) }
                    },
                    move |result| Message::GotConversations(team_id_clone.clone(), result),
                )
            }
            Message::GotConversations(team_id, conversations) => {
                self.team_conversations
                    .insert(team_id, conversations.unwrap());
                snap_to(Id::new("conversation_column"), RelativeOffset::END)
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
        .font(include_bytes!("../resources/Twemoji-15.1.0.ttf").as_slice())
        .run_with(Counter::new)
}
