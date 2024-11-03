use iced::{Color, Element, Task, Theme};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

mod api;
use api::{gen_refresh_token_from_code, gen_tokens, user_teams, AccessToken, ApiError};

mod pages;
use pages::{homepage, login, team};

#[derive(Serialize, Deserialize, Debug)]
struct Counter {
    cache: AppCache,
    page: Page,
    search_teams_input_value: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum Page {
    Login,
    Homepage,
    Team,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppCache {
    refresh_token: AccessToken,
    access_tokens: HashMap<String, AccessToken>,
    user_teams: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum ApiCalls {
    UserTeams,
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    Authorized(Result<AccessToken, ApiError>),
    GenRefreshToken(AuthorizationCodes),
    SetRefreshToken(Result<AccessToken, ApiError>),
    SetTokens(Result<AccessToken, ApiError>, String, ApiCalls),
    SavedCache(()),
    UserTeamsFetched(Result<HashMap<String, Value>, String>),
    Join,
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
    let json = serde_json::to_string(&cache).unwrap();
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
            user_teams: HashMap::new(),
        };

        let file_path = "app.json";
        if Path::new(file_path).exists() {
            // Read the file if it exists
            let file_content = fs::read_to_string(file_path).unwrap();
            cache = serde_json::from_str(&file_content).unwrap()
        }

        if cache.refresh_token.expires > get_epoch_s() {
            let teams_scope = "https://chatsvcagg.teams.microsoft.com/.default";
            // Check if the token is expired or not. Before. TODO
            let refresh_token = cache.refresh_token.clone();

            return (
                Self {
                    page: Page::Homepage,
                    cache: cache.clone(),
                    search_teams_input_value: "".to_string(),
                },
                Task::perform(gen_tokens(refresh_token, teams_scope), |response| {
                    Message::SetTokens(response, teams_scope.to_string(), ApiCalls::UserTeams)
                }),
            );
        }

        (
            Self {
                page: Page::Login,
                cache: cache,
                search_teams_input_value: "".to_string(),
            },
            Task::perform(authorize(), |response| response).then(|response| {
                Task::perform(
                    gen_refresh_token_from_code(response.code, response.code_verifier),
                    Message::Authorized,
                )
            }),
        )
    }
    fn view(&self) -> Element<Message> {
        match self.page {
            Page::Login => login(),
            Page::Homepage => homepage(
                self.cache.user_teams.clone(),
                self.search_teams_input_value.clone(),
            ),
            Page::Team => team(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Authorized(response) => {
                if let Ok(refresh_token) = response {
                    self.cache.refresh_token = refresh_token.clone();
                    self.page = Page::Homepage;
                    let teams_scope = "https://chatsvcagg.teams.microsoft.com/.default";
                    Task::perform(gen_tokens(refresh_token, teams_scope), |response| {
                        Message::SetTokens(response, teams_scope.to_string(), ApiCalls::UserTeams)
                    })
                } else {
                    println!("Error occurred authorizing.");
                    Task::none()
                }
            }

            Message::GenRefreshToken(code) => Task::perform(
                gen_refresh_token_from_code(code.code, code.code_verifier),
                Message::SetRefreshToken,
            ),
            Message::SetRefreshToken(response) => {
                if let Ok(refresh_token) = response {
                    self.cache.refresh_token = refresh_token;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::none()
            }

            Message::SetTokens(response, scope, callback) => {
                if let Ok(access_token) = response {
                    self.cache.access_tokens.insert(scope, access_token.clone());
                    match callback {
                        ApiCalls::UserTeams => {
                            Task::perform(user_teams(access_token), Message::UserTeamsFetched)
                        }
                        ApiCalls::None => Task::none(),
                    }
                } else {
                    println!("Error occurred generating token");
                    Task::none()
                }
            }

            Message::SavedCache(_response) => Task::none(),

            Message::UserTeamsFetched(response) => {
                if let Ok(user_teams) = response {
                    self.cache.user_teams = user_teams;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::perform(save_cache(self.cache.clone()), Message::SavedCache)
            }

            Message::Join => {
                println!("Join button pressed");
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
