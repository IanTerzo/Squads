use iced::{Color, Element, Task, Theme};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fs, io::Write, path::Path};

use webbrowser;

mod components;
use components::cached_image::save_cached_image;

mod utils;
mod widgets;

mod api;
use api::{
    authorize_avatar, authorize_image, authorize_profile_picture, authorize_team_picture,
    fetch_short_profile, team_conversations, user_details, user_properties, AccessToken, Chat,
    ShortProfile, Team, TeamConversations,
};

mod auth;
use auth::{get_or_gen_skype_token, get_or_gen_token};
mod pages;
use pages::app;
use pages::page_chat::chat;
use pages::page_home::home;
use pages::page_login::login;
use pages::page_team::team;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppCache {
    access_tokens: HashMap<String, AccessToken>,
    user_id: String,
    org_users: HashMap<String, ShortProfile>,
    teams: Vec<Team>,
    chats: Vec<Chat>,
    team_conversations: HashMap<String, TeamConversations>, // String is the team id
}

#[derive(Debug, Clone)]
struct Counter {
    page: Page,
    reply_options: HashMap<String, bool>, // String is the conversation id
    cache: Arc<Mutex<AppCache>>,
    history: Vec<Page>,
    emoji_map: HashMap<String, String>,
    search_teams_input_value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Authorized(()),
    DoNothing(()),
    LinkClicked(String),
    Join,
    Jump(Page),
    ToggleReplyOptions(String),
    HistoryBack,
    OpenTeam(String, String),
    FetchTeamImage(String, String, String),
    FetchUserImage(String, String),
    FetchAvatar(String),
    AuthorizeImage(String),
    ShowConversations(()),
    GotConversations(String, Result<TeamConversations, String>),
    ContentChanged(String),
}

fn get_chat_users_mri(chats: Vec<Chat>) -> Vec<String> {
    let mut user_mri = HashSet::new();
    for chat in chats {
        for member in chat.members {
            user_mri.insert(member.mri);
        }
    }
    user_mri.into_iter().collect()
}

async fn save_cache(cache: AppCache) {
    let json = serde_json::to_string_pretty(&cache).unwrap();
    let mut file = fs::File::create("app.json").unwrap(); // Should be inside config folder instead.
    file.write_all(json.as_bytes()).unwrap();
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        let mut cache = AppCache {
            access_tokens: HashMap::new(),
            user_id: "".to_string(),
            org_users: HashMap::new(),
            teams: Vec::new(),
            chats: Vec::new(),
            team_conversations: HashMap::new(),
        };

        let file_path = "app.json";

        if Path::new(file_path).exists() {
            let file_content = fs::read_to_string(file_path).unwrap();
            cache = serde_json::from_str(&file_content).unwrap()
        }

        let cache_mutex = Arc::new(Mutex::new(cache.clone()));

        let file_content = fs::read_to_string("emojies.json").unwrap();
        let emojies: HashMap<String, String> = serde_json::from_str(&file_content).unwrap();

        let mut counter_self = Self {
            page: Page {
                view: View::Login,
                current_team_id: "0".to_string(),
                current_channel_id: "0".to_string(),
                show_conversations: false,
            },
            reply_options: HashMap::new(),
            history: Vec::new(),
            emoji_map: emojies,
            cache: cache_mutex.clone(),
            search_teams_input_value: "".to_string(),
        };

        //if cache.refresh_token.expires < get_epoch_s() {} show login page

        counter_self.page.view = View::Homepage;
        counter_self.history.push(counter_self.clone().page);

        (
            counter_self,
            Task::perform(
                async move {
                    let cache_mutex = Arc::clone(&cache_mutex);

                    let access_token_details = get_or_gen_token(
                        cache_mutex.clone(),
                        "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                    );

                    let user_details = user_details(access_token_details.clone()).unwrap();
                    cache_mutex.lock().unwrap().teams = user_details.clone().teams;
                    cache_mutex.lock().unwrap().chats = user_details.clone().chats;

                    let user_mris = get_chat_users_mri(user_details.chats);

                    let user_short_profiles =
                        fetch_short_profile(access_token_details.clone(), user_mris);

                    let mut profile_map = HashMap::new();

                    for short_profile in user_short_profiles.unwrap().value {
                        profile_map.insert(short_profile.clone().mri, short_profile);
                    }

                    cache_mutex.lock().unwrap().org_users = profile_map;

                    let access_token_properties = get_or_gen_token(
                        cache_mutex.clone(),
                        "https://ic3.teams.office.com/.default".to_string(),
                    );

                    let user_properties = user_properties(access_token_properties.clone()).unwrap();
                    cache_mutex.lock().unwrap().user_id =
                        user_properties.primary_member_name.unwrap();
                },
                Message::Authorized,
            ),
        )
    }

    fn view(&self) -> Element<Message> {
        println!("view called");

        match self.page.view {
            View::Login => app(login()),
            View::Homepage => app(home(
                self.cache.lock().unwrap().teams.clone(),
                self.search_teams_input_value.clone(),
            )),
            View::Team => {
                let cache = self.cache.lock().unwrap();

                let current_team = cache
                    .teams
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

                let emoji_map = &self.emoji_map;
                // NOTE: We need to open the team page withou any conversations first, and the load the conversations, otherwise the app would feel unresposive if it froze until the conversations where loaded (and rendered into iced components)
                // That's why this exists. Better solutions are welcome.

                if self.page.show_conversations {
                    let conversation = cache.team_conversations.get(&self.page.current_team_id);
                    app(team(
                        current_team,
                        current_channel,
                        conversation.cloned(),
                        reply_options,
                        emoji_map,
                    ))
                } else {
                    app(team(
                        current_team,
                        current_channel,
                        None,
                        reply_options,
                        emoji_map,
                    ))
                }
            }
            View::Chat => {
                let cache = self.cache.clone().lock().unwrap().clone();
                app(chat(cache.chats, cache.org_users, cache.user_id))
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DoNothing(_) => Task::none(),

            Message::Authorized(_response) => {
                self.page.view = View::Homepage;
                self.history.push(self.page.clone());
                Task::perform(
                    save_cache(self.cache.lock().unwrap().clone()),
                    Message::DoNothing,
                )
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

            Message::FetchTeamImage(picture_e_tag, group_id, display_name) => {
                let cache_mutex = self.cache.clone();

                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            cache_mutex,
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

                            save_cached_image(picture_e_tag, bytes);
                        }
                    },
                    Message::DoNothing,
                )
            }

            Message::FetchUserImage(user_id, display_name) => {
                let cache_mutex = self.cache.clone();
                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            cache_mutex,
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                        );
                        async {
                            let user_id = user_id;

                            let bytes = authorize_profile_picture(
                                access_token,
                                user_id.clone(),
                                display_name,
                            )
                            .unwrap();

                            save_cached_image(user_id, bytes);
                        }
                    },
                    Message::DoNothing,
                )
            }

            Message::FetchAvatar(url) => {
                println!("1fetching..");

                let cache_mutex = self.cache.clone();
                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            cache_mutex.clone(),
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                        );

                        let skype_token = get_or_gen_skype_token(cache_mutex, access_token);

                        async {
                            let url = url;

                            let identifier = url
                                .clone()
                                .replace(
                                    "https://eu-prod.asyncgw.teams.microsoft.com/v1/objects",
                                    "",
                                )
                                .replace("/", "");

                            let bytes = authorize_avatar(skype_token, url.clone()).unwrap();

                            save_cached_image(identifier, bytes);
                        }
                    },
                    Message::DoNothing,
                )
            }

            Message::AuthorizeImage(image_id) => {
                let cache_mutex = self.cache.clone();

                Task::perform(
                    {
                        let access_token = get_or_gen_token(
                            cache_mutex.clone(),
                            "https://api.spaces.skype.com/Authorization.ReadWrite".to_string(),
                        );

                        let skype_token = get_or_gen_skype_token(cache_mutex, access_token);

                        async {
                            let image_id = image_id;

                            let bytes = authorize_image(skype_token, image_id.clone()).unwrap();

                            save_cached_image(image_id, bytes);
                        }
                    },
                    Message::DoNothing,
                )
            }

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

                let cache_mutex = self.cache.clone();

                let team_id = self.page.current_team_id.clone();
                let channel_id = self.page.current_channel_id.clone();
                let team_id_clone = self.page.current_team_id.clone();

                Task::perform(
                    async move {
                        let access_token = get_or_gen_token(
                            cache_mutex,
                            "https://chatsvcagg.teams.microsoft.com/.default".to_string(),
                        );
                        team_conversations(access_token, team_id, channel_id)
                    },
                    move |result| Message::GotConversations(team_id_clone.clone(), result),
                )
            }
            Message::GotConversations(team_id, conversations) => {
                let mut cache_mutex = self.cache.lock().unwrap();
                cache_mutex
                    .team_conversations
                    .insert(team_id, conversations.unwrap());
                Task::perform(save_cache(cache_mutex.clone()), Message::DoNothing)
            }
            Message::ContentChanged(content) => {
                self.search_teams_input_value = content;
                Task::none()
            }
        }
    }

    fn theme(&self) -> Theme {
        let custom_palette = iced::theme::palette::Palette {
            background: Color::parse("#3c3c3c").expect("Background color is invalid."),
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
        .theme(Counter::theme)
        .run_with(Counter::new)
}
