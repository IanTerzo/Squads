use std::{
    process::{Child, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use directories::ProjectDirs;
use tokio::time::sleep;

use std::sync::{Arc, Mutex};

use rand::Rng;
use sha2::{Digest, Sha256};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::api::{gen_refresh_token_from_code, gen_skype_token, gen_tokens, AccessToken};
use crate::AppCache;

use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::runtime::Builder;
use url::form_urlencoded;

fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

const CHROMEDRIVER_PORT: u16 = 35101;

fn start_chromedriver(port: u16) -> std::io::Result<Child> {
    Command::new("chromedriver")
        .arg(format!("--port={}", port))
        .stdout(std::process::Stdio::null()) // Redirect output if needed
        .stderr(std::process::Stdio::null())
        .spawn()
}

async fn get_auth_code(challenge: String) -> WebDriverResult<String> {
    let base_url = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";

    let mut params = vec![
        ("client_id", "5e3ce6c0-2b1f-4285-8d4b-75ee78787346"),
        ("scope", "openId profile openid offline_access"),
        ("redirect_uri", "https://teams.microsoft.com/v2"),
        ("response_mode", "fragment"),
        ("response_type", "code"),
        ("x-client-SKU", "msal.js.browser"),
        ("x-client-VER", "3.18.0"),
        ("client_info", "1"),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "plain"),
        ("prompt", "none"),
    ];

    // Build initial auth URL
    let encoded_params = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&params)
        .finish();
    let auth_url = format!("{}?{}", base_url, encoded_params);

    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");
    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    cache_dir.push("chrome-data");
    let dir = cache_dir.to_str().unwrap();

    let mut chrome_options = DesiredCapabilities::chrome();
    chrome_options.add_arg(&format!("--app={}", auth_url))?;
    chrome_options.add_arg(&format!("--user-data-dir={}", dir))?;
    chrome_options.add_arg("--window-size=550,500")?;
    chrome_options.add_arg("--disable-infobars")?;
    chrome_options.add_experimental_option("excludeSwitches", vec!["enable-automation"])?;

    // Start WebDriver

    let driver = WebDriver::new(
        format!("http://localhost:{CHROMEDRIVER_PORT}"),
        chrome_options,
    )
    .await?;

    loop {
        sleep(Duration::from_millis(250)).await;

        let current_url = driver.current_url().await?.to_string();
        if current_url.contains("https://teams.microsoft.com/v2/#code=") {
            let code = current_url
                .split("code=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .unwrap();

            driver.quit().await?;
            return Ok(code.to_string());
        } else if current_url.contains("https://teams.microsoft.com/v2/#error=interaction_require")
        {
            params.retain(|(k, _)| *k != "prompt");
            let encoded_params = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&params)
                .finish();
            let auth_url = format!("{}?{}", base_url, encoded_params);

            driver.goto(&auth_url).await?;
            continue;
        }
    }
}

fn gen_code_challenge() -> String {
    let code_verifier: String = (0..64)
        .map(|_| rand::rng().random_range(33..=126) as u8 as char)
        .collect();

    let hash = Sha256::digest(code_verifier.as_bytes());

    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    code_challenge
}

pub struct AuthorizationCode {
    pub code: String,
    pub code_verifier: String,
}

pub fn authorize() -> Result<AuthorizationCode, String> {
    let challenge = "lXHr5Zb7Mro-sKjZXn5xYpYhMX3ik5MsA9APHPlDtpQ".to_string();

    let rt = Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .map_err(|e| format!("Failed to build runtime: {:?}", e))?;

    let mut chromedriver = start_chromedriver(CHROMEDRIVER_PORT)
        .map_err(|e| format!("Failed to start chromedriver: {:?}", e))?;

    let code = rt
        .block_on(get_auth_code(challenge.clone()))
        .map_err(|e| format!("Error while getting auth code: {:?}", e));

    chromedriver
        .kill()
        .map_err(|e| format!("Failed to kill chromedriver: {:?}", e))?;

    code.map(|code| AuthorizationCode {
        code,
        code_verifier: challenge,
    })
}

pub fn get_or_gen_token(cache: Arc<Mutex<AppCache>>, scope: String) -> AccessToken {
    let refresh_token = cache
        .lock()
        .unwrap()
        .access_tokens
        .entry("refresh_token".to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                let auth_code = authorize().unwrap();
                *token =
                    gen_refresh_token_from_code(auth_code.code, auth_code.code_verifier).unwrap()
            }
        })
        .or_insert_with(|| {
            let auth_code = authorize().unwrap();
            gen_refresh_token_from_code(auth_code.code, auth_code.code_verifier).unwrap()
        })
        .clone();

    cache
        .lock()
        .unwrap()
        .access_tokens
        .entry(scope.to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                *token = gen_tokens(refresh_token.clone(), scope.to_string()).unwrap();
            }
        })
        .or_insert_with(|| gen_tokens(refresh_token, scope.to_string()).unwrap())
        .clone()
}

pub fn get_or_gen_skype_token(
    cache: Arc<Mutex<AppCache>>,
    access_token: AccessToken,
) -> AccessToken {
    cache
        .lock()
        .unwrap()
        .access_tokens
        .entry("skype_token".to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                *token = gen_skype_token(access_token.clone()).unwrap();
            }
        })
        .or_insert_with(|| gen_skype_token(access_token).unwrap())
        .clone()
}
