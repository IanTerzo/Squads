use bytes::Bytes;
use iced::{Color, Element, Task, Theme};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    ApiError, Team, UserDetails,
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

    SetRefreshToken(Result<AccessToken, ApiError>),
    SetTokens(Result<AccessToken, ApiError>, String),
    SavedCache(()),
    UserDetailsFetched(Result<UserDetails, String>),
    Join,
    HistoryBack,
    OpenTeam(String),
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

impl Counter {
    fn new() -> (Self, Task<Message>) {
        let mut cache = AppCache {
            refresh_token: AccessToken {
                value: "".to_string(),
                expires: 0,
            },
            access_tokens: HashMap::new(),
            teams: Vec::new(),
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
                    let teams_token = gen_tokens(refresh_token, scope.to_string()).await.unwrap();

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

                        let refresh_token = cache_mutex.lock().unwrap().refresh_token.clone();

                        let scope = "https://chatsvcagg.teams.microsoft.com/.default";
                        let teams_token =
                            gen_tokens(refresh_token, scope.to_string()).await.unwrap();

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
            View::Team => app(team_page(self.cache.lock().unwrap().teams.iter().find(|team| team.id == self.page.current_team_id).unwrap().clone()) // Long ass line
            ),
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
                                gen_tokens(refresh_token, scope.to_string()).await.unwrap();

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

            Message::SetRefreshToken(response) => {
                if let Ok(refresh_token) = response {
                    self.cache.lock().unwrap().refresh_token = refresh_token;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::none()
            }

            Message::SetTokens(response, scope) => {
                println!("{:#?}", self.cache.lock().unwrap().access_tokens);
                if let Ok(access_token) = response {
                    self.cache
                        .lock()
                        .unwrap()
                        .access_tokens
                        .insert(scope, access_token.clone());
                    println!("{:#?}", self.cache.lock().unwrap().access_tokens);
                    Task::none()
                } else {
                    println!("Error occurred generating token");
                    Task::none()
                }
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
            Message::HistoryBack => {

                self.page = self.history[0].clone(); // WILL FIX SOON!
                Task::none()
            }
            Message::OpenTeam(id) => {
                let team_page = Page {
                    view : View::Team,
                    current_team_id: id.clone(),
                    current_channel_id: id,
                };
                self.page = team_page.clone();
                self.history.push(team_page);
                println!("OpenTeam button pressed");
                Task::none()
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
    iced::application("Styling - Iced", Counter::update, Counter::view)
        .theme(Counter::theme)
        .run_with(Counter::new)
}
