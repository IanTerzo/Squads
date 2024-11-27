use bytes::Bytes;
use iced::{Color, Element, Task, Theme};
use iced::widget::markdown;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use htmd::HtmlToMarkdown;

use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    fs,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

mod api;
use api::{
    authorize_team_picture, gen_refresh_token_from_code, gen_tokens, user_details, AccessToken,
    ApiError, Team, UserDetails, team_conversations, TeamConversations
};

mod pages;
use pages::{app, homepage, login, team_page};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum View {
    Login,
    Homepage,
    Team,
}

// Any information needed to display the current page
#[derive(Debug, Clone)]
struct Page {
    view: View,
    current_team_id: String,
    current_channel_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppCache {
    refresh_token: AccessToken,
    access_tokens: HashMap<String, AccessToken>,
    teams: Vec<Team>,
    team_conversations: HashMap<String, TeamConversations> // String is the team id
}

#[derive(Debug, Clone)]
struct Counter {
    cache: Arc<Mutex<AppCache>>,
    page: Page,
    history: Vec<Page>,
    search_teams_input_value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TeamPictureFetched(Bytes, String),
    Authorized(UserDetails),
    SavedCache(()),
    LinkClicked(markdown::Url),
    UserDetailsFetched(Result<UserDetails, String>),
    Join,
    HistoryBack,
    OpenTeam(String, String),
    GotConversations(String, Result<TeamConversations, String>),
    ContentChanged(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct AuthorizationCodes {
    code: String,
    code_verifier: String,
}

async fn authorize() -> AuthorizationCodes {
    let output = Command::new("python3")
        .arg("auth.py")
        .output()
        .expect("Failed to execute command");

    let json_data = String::from_utf8(output.stdout).expect("Found invalid UTF-8");
    let codes_parsed: AuthorizationCodes = serde_json::from_str(json_data.as_str()).unwrap();
    codes_parsed
}

async fn save_cache(cache: AppCache) {
    let json = serde_json::to_string_pretty(&cache).unwrap();
    let mut file = fs::File::create("app.json").unwrap(); // Should be inside config folder instead.
    file.write_all(json.as_bytes()).unwrap();
}

fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


fn get_or_gen_token(mut cache: Arc<Mutex<AppCache>>, scope: String) -> AccessToken{
    let refresh_token = cache.lock().unwrap().refresh_token.clone();

    cache.lock().unwrap().access_tokens.entry(scope.to_string()).and_modify(|token| {
        if token.expires <  get_epoch_s(){
            *token = gen_tokens(refresh_token.clone(), scope.to_string()).unwrap();
        }
    }).or_insert_with(|| {
        gen_tokens(refresh_token, scope.to_string()).unwrap()
    }).clone()
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        let mut cache = AppCache {
            refresh_token: AccessToken {
                value: "".to_string(),
                expires: 0,
            },
            access_tokens: HashMap::new(),
            teams: Vec::new(),
            team_conversations: HashMap::new()
        };

        let file_path = "app.json";

        if Path::new(file_path).exists() {
            let file_content = fs::read_to_string(file_path).unwrap();
            cache = serde_json::from_str(&file_content).unwrap()
        }

        let cache_mutex = Arc::new(Mutex::new(cache.clone()));

        let mut counter_self = Self {
            page: Page {
                view: View::Login,
                current_team_id: "0".to_string(),
                current_channel_id: "0".to_string(),
            },
            history: Vec::new(),
            cache: cache_mutex.clone(),
            search_teams_input_value: "".to_string(),
        };

        if cache.refresh_token.expires < get_epoch_s() {
            return (
                counter_self,
                Task::perform(
                    async move {
                        let cache_mutex = Arc::clone(&cache_mutex);

                        let authorization_codes = authorize().await;
                        let refresh_token = gen_refresh_token_from_code(
                            authorization_codes.code,
                            authorization_codes.code_verifier,
                        )
                        .await
                        .unwrap();

                        cache_mutex.lock().unwrap().refresh_token = refresh_token.clone();

                        let scope = "https://chatsvcagg.teams.microsoft.com/.default";
                        let teams_token =
                            gen_tokens(refresh_token, scope.to_string()).unwrap();

                        cache_mutex
                            .lock()
                            .unwrap()
                            .access_tokens
                            .insert(scope.to_string(), teams_token.clone());

                        let user_details = user_details(teams_token.clone()).await.unwrap();

                        cache_mutex.lock().unwrap().teams = user_details.clone().teams;

                        user_details
                    },
                    |response| Message::Authorized(response),
                ),
            );
        }

        counter_self.page.view = View::Homepage;
        counter_self.history.push(counter_self.clone().page);

        (
            counter_self,
            Task::perform(
                async move {
                    let cache_mutex = Arc::clone(&cache_mutex);

                    let access_token = get_or_gen_token(cache_mutex.clone(), "https://chatsvcagg.teams.microsoft.com/.default".to_string());


                    let user_details = user_details(access_token.clone()).await.unwrap();

                    cache_mutex.lock().unwrap().teams = user_details.clone().teams;

                    user_details
                },
                |response| Message::Authorized(response),
            ),
        )
    }

    fn view(&self) -> Element<Message> {
        println!("view called");
        match self.page.view {
            View::Login => app(login()),
            View::Homepage => app(homepage(
                self.cache.lock().unwrap().teams.clone(),
                self.search_teams_input_value.clone(),
            )),
            View::Team => {
                let cache = self
                    .cache
                    .lock()
                    .unwrap();

                let team =
                    cache
                    .teams
                    .iter()
                    .find(|team| team.id == self.page.current_team_id)
                    .unwrap()
                    .clone();

                let channel = team.channels.iter().find(|channel| channel.id == self.page.current_channel_id).unwrap().clone();

                let conversation = cache.team_conversations.get(&self.page.current_team_id);
                app(team_page(team, channel, conversation.cloned()))
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Authorized(response) => {
                // Summary:
                // Perform a task to get the required access token and set it, then -> run a batch of tasks for all the required team pictures -> save the cache

                self.page.view = View::Homepage;
                self.history.push(self.page.clone());

                let cache_mutex = self.cache.clone();
                Task::perform(
                    {
                        let cache_mutex = Arc::clone(&cache_mutex);
                        async move {
                            let refresh_token = cache_mutex.lock().unwrap().refresh_token.clone();

                            let scope = "https://api.spaces.skype.com/Authorization.ReadWrite";
                            let teams_token =
                                gen_tokens(refresh_token, scope.to_string()).unwrap();

                            cache_mutex
                                .lock()
                                .unwrap()
                                .access_tokens
                                .insert(scope.to_string(), teams_token.clone());

                            teams_token
                        }
                    },
                    |response| response,
                )
                .then({
                    let cache_mutex = Arc::clone(&self.cache);
                    move |access_token| {
                        let mut picture_tasks = Vec::new();

                        for team in cache_mutex.lock().unwrap().teams.clone() {
                            picture_tasks.push(Task::perform(
                                {
                                    let access_token = access_token.clone();
                                    let team = team.clone();
                                    async {
                                        let picture_e_tag = team.picture_e_tag;
                                        (
                                            authorize_team_picture(
                                                access_token,
                                                team.team_site_information.group_id,
                                                picture_e_tag.clone(),
                                                team.display_name,
                                            )
                                            .await
                                            .unwrap(),
                                            picture_e_tag,
                                        )
                                    }
                                },
                                |(bytes, picture_e_tag)| {
                                    Message::TeamPictureFetched(bytes, picture_e_tag)
                                },
                            ))
                        }

                        Task::batch(picture_tasks).chain(Task::perform(
                            save_cache(cache_mutex.lock().unwrap().clone()),
                            Message::SavedCache,
                        ))
                    }
                })
            }

            Message::TeamPictureFetched(bytes, picture_e_tag) => {
                let filename = format!("image-cache/{}.jpeg", picture_e_tag);

                if !Path::new(&filename).exists() {
                    let mut file = File::create(filename).unwrap();
                    let _ = file.write_all(&bytes);
                }

                Task::none()
            }

            Message::SavedCache(_response) => Task::none(),

            Message::UserDetailsFetched(response) => {
                if let Ok(user_details) = response {
                    self.cache.lock().unwrap().teams = user_details.teams;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::perform(
                    save_cache(self.cache.lock().unwrap().clone()),
                    Message::SavedCache,
                )
            }

            Message::Join => {
                println!("Join button pressed");
                Task::none()
            }
            Message::LinkClicked(url) => {
                println!("The following url was clicked: {url}");
                Task::none()
            }
            Message::HistoryBack => {
                self.page = self.history[0].clone(); // WILL FIX SOON!
                Task::none()
            }
            Message::OpenTeam(team_id, channel_id) => {
                let team_page = Page {
                    view: View::Team,
                    current_team_id: team_id.clone(),
                    current_channel_id: channel_id.clone(),
                };
                self.page = team_page.clone();
                self.history.push(team_page);
                println!("OpenTeam button pressed");


                let cache_mutex = self.cache.clone();

                let team_id_clone = team_id.clone();
                Task::perform(async move {
                    let access_token = get_or_gen_token(cache_mutex, "https://chatsvcagg.teams.microsoft.com/.default".to_string());
                    team_conversations(access_token, team_id, channel_id).await
                }, move |result| Message::GotConversations(team_id_clone.clone(), result))

            }
            Message::GotConversations(team_id, conversations) => {
                let mut cache_mutex = self.cache.lock().unwrap();
                cache_mutex.team_conversations.insert(team_id, conversations.unwrap());
                Task::perform(
                    save_cache(cache_mutex.clone()),
                    Message::SavedCache,
                )
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
