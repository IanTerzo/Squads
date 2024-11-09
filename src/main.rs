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
use std::sync::{Arc, Mutex};

mod api;
use api::{gen_refresh_token_from_code, gen_tokens, user_details, AccessToken, ApiError, UserDetails, authorize_team_picture};

mod pages;
use pages::{homepage, login, team};

#[derive( Debug)]
struct Counter {
    cache: Arc<Mutex<AppCache>>,
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
    teams: Vec<HashMap<String, Value>>
}



#[derive(Debug, Clone)]
pub enum Message {
    Authorized(Result<AccessToken, ApiError>),
    GenRefreshToken(AuthorizationCodes),
    SetRefreshToken(Result<AccessToken, ApiError>),
    SetTokens(Result<AccessToken, ApiError>, String),
    SavedCache(()),
    UserDetailsFetched(Result<UserDetails, String>),
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
            teams : Vec::new()
        };

        let file_path = "app.json";
        if Path::new(file_path).exists() {
            // TODO, Check if the file exists
            let file_content = fs::read_to_string(file_path).unwrap();
            cache = serde_json::from_str(&file_content).unwrap()
        }

        let cache_mutex = Arc::new(Mutex::new(cache.clone()));
        println!("{:#?}", cache.access_tokens.clone());
        if cache.refresh_token.expires > get_epoch_s() {           
            let refresh_token = cache.refresh_token.clone();
            
            let mut needed_tokens = Vec::new();

            let teams_scope = "https://chatsvcagg.teams.microsoft.com/.default";
            let token_expired = cache.access_tokens
                .get(teams_scope)
                .map_or(true, |token| token.expires <= get_epoch_s());
        
            if token_expired {
                needed_tokens.push(Task::perform({ let cache_mutex = Arc::clone(&cache_mutex); async move {
                   
                    let response = gen_tokens(refresh_token.clone(), teams_scope.to_string()).await.unwrap(); 
                    cache_mutex.lock().unwrap().access_tokens.insert(teams_scope.to_string(), response.clone());
                    user_details(response.clone()).await
                }}, Message::UserDetailsFetched));
            }
            else {
                needed_tokens.push(Task::perform(user_details(cache.access_tokens[teams_scope].clone()), Message::UserDetailsFetched))
            }
            
            let refresh_token_clone = cache.refresh_token.clone();

            return (
                Self {
                    page: Page::Homepage,
                    cache: cache_mutex,   
                    search_teams_input_value: "".to_string(),
                },
                Task::batch(needed_tokens)
                
            );
        }

        (
            Self {
                page: Page::Login,
                cache: cache_mutex.clone(),
                search_teams_input_value: "".to_string(),
            },
            Task::perform(async { 
                let authorization_codes = authorize().await;
                gen_refresh_token_from_code(authorization_codes.code, authorization_codes.code_verifier).await.unwrap()
            }, |response| response)
            .then( { 
                move |response| {
                    let cache_mutex = Arc::clone(&cache_mutex);
                    cache_mutex.lock().unwrap().refresh_token = response.clone();
                    let scope = "https://chatsvcagg.teams.microsoft.com/.default";
                    Task::perform(gen_tokens(response, scope.to_string()), |response| response)
                    .and_then({
                        let cache_mutex =  Arc::clone(&cache_mutex); move |response| {
                            cache_mutex.lock().unwrap().access_tokens.insert(scope.to_string(), response.clone());
                            Task::perform(user_details(response ), Message::UserDetailsFetched)
                        }
                        }) 
                }}
                
            ),
        )
    }
    fn view(&self) -> Element<Message> {
        println!("view called");
        match self.page {
            Page::Login => login(),
            Page::Homepage => homepage(
                self.cache.lock().unwrap().teams.clone(),
                self.search_teams_input_value.clone(),
            ),
            Page::Team => team(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Authorized(response) => {
               
                if let Ok(r_refresh_token) = response {
                    Task::none()


                       
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
                    self.cache.lock().unwrap().refresh_token = refresh_token;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::none()
            }

            Message::SetTokens(response, scope) => {
                println!("{:#?}", self.cache.lock().unwrap().access_tokens);
                if let Ok(access_token) = response {
                    self.cache.lock().unwrap().access_tokens.insert(scope, access_token.clone());
                    println!("{:#?}", self.cache.lock().unwrap().access_tokens);
                    Task::none()
                
                } else {
                    println!("Error occurred generating token");
                    Task::none()
                }
            }

            Message::SavedCache(_response) => Task::none(),

            Message::UserDetailsFetched(response) => {
                self.page = Page::Homepage;

                if let Ok(user_details) = response {
                    self.cache.lock().unwrap().teams = user_details.teams;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::perform(save_cache(self.cache.lock().unwrap().clone()), Message::SavedCache)
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
