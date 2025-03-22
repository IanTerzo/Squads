use crate::api::{gen_refresh_token_from_code, gen_skype_token, gen_tokens, AccessToken};
use crate::utils::get_cache;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ipc_channel::ipc::IpcOneShotServer;
use rand::Rng;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::sync::Arc;
use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};
use url::form_urlencoded;
extern crate reqwest;
use reqwest::header::{self, SET_COOKIE};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizationCode {
    pub code: String,
    pub code_verifier: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthorizationInfo {
    authorization_codes: Option<AuthorizationCode>,
    cookies: Option<Vec<Cookie>>,
    success: bool,
}

// This struct does not cover everything given by the api
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OauthCredentialInfo {
    pub url_login: String,
}

#[derive(Debug, Deserialize)]
pub struct ReprocessInfo {
    url_login: String,
    light: String,
    persistent: String,
}

fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn gen_code_challenge() -> String {
    let code_verifier: String = (0..64)
        .map(|_| rand::rng().random_range(33..=126) as u8 as char)
        .collect();

    let hash = Sha256::digest(code_verifier.as_bytes());

    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    code_challenge
}

pub fn get_reprocess_url(
    code_challenge: String,
    ests_auth_persistant_token: String,
) -> Result<ReprocessInfo, String> {
    let base_url = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";

    let params = vec![
        ("client_id", "5e3ce6c0-2b1f-4285-8d4b-75ee78787346"),
        ("scope", "openId profile openid offline_access"),
        ("redirect_uri", "https://teams.microsoft.com/v2"),
        ("response_mode", "fragment"),
        ("response_type", "code"),
        ("x-client-SKU", "msal.js.browser"),
        ("x-client-VER", "3.18.0"),
        ("client_info", "1"),
        ("code_challenge", code_challenge.as_str()),
        ("code_challenge_method", "plain"),
        ("prompt", "none"),
    ];

    let encoded_params = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&params)
        .finish();
    let auth_url = format!("{}?{}", base_url, encoded_params);

    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::COOKIE,
        format!("ESTSAUTHPERSISTENT={};", ests_auth_persistant_token)
            .parse()
            .unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client.get(auth_url).headers(headers).send().unwrap();

    // Get the relevant tokens from cookies

    let mut ests_auth_persistent_token = "".to_string();
    let mut ests_auth_light_token = "".to_string();

    if let Some(_set_cookie_headers) = res.headers().get_all(SET_COOKIE).iter().next() {
        for header_value in res.headers().get_all(SET_COOKIE).iter() {
            let header_value = header_value.to_str().unwrap().split(";").next().unwrap();
            if header_value.starts_with("ESTSAUTHPERSISTENT=") {
                ests_auth_persistent_token = header_value
                    .strip_prefix("ESTSAUTHPERSISTENT=")
                    .unwrap()
                    .to_string();
            } else if header_value.starts_with("ESTSAUTHLIGHT=") {
                ests_auth_light_token = header_value
                    .strip_prefix("ESTSAUTHLIGHT=")
                    .unwrap()
                    .to_string();
            }
        }
    } else {
        return Err("No Set-Cookie header found.".to_string());
    }

    if ests_auth_persistent_token == "" {
        return Err("No response ests auth persistent token found.".to_string());
    }
    if ests_auth_light_token == "" {
        return Err("No response ests auth light token found.".to_string());
    }

    // Get the login url from the response HTML

    let res_text = res.text().unwrap();

    let document = Html::parse_document(&res_text);
    let selector = Selector::parse("title").unwrap();

    let title = document
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>());

    if let Some(title) = title {
        // If the request was sucessfull redirecting will be the title.
        if title == "Redirecting" {
            let selector = Selector::parse("script").unwrap();

            let first_script = document.select(&selector).next().unwrap();
            let contents = first_script.text().collect::<String>();
            let cleaned_contents = contents
                .strip_prefix("//<![CDATA[\n$Config=")
                .unwrap()
                .strip_suffix(";\n//]]>")
                .unwrap();

            return match serde_json::from_str::<OauthCredentialInfo>(cleaned_contents) {
                Ok(parsed) => Ok(ReprocessInfo {
                    url_login: parsed.url_login,
                    light: ests_auth_light_token,
                    persistent: ests_auth_persistent_token,
                }),
                Err(e) => Err(format!("Failed to parse JSON: {}", e).to_string()),
            };
        }
    }

    Err(format!(
        "Response HTML does not have \"Redirecting\" as title. Response body: {}",
        res_text
    )
    .to_string())
}

pub fn get_authorization_code_from_redirect(
    login_url: String,
    ests_auth_persistent_token: String,
) -> Result<String, String> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::COOKIE,
        format!("ESTSAUTHPERSISTENT={};", ests_auth_persistent_token)
            .parse()
            .unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client.get(&login_url).headers(headers).send().unwrap();
    if res.status().is_redirection() {
        if let Some(location) = res.headers().get(header::LOCATION) {
            Ok(location
                .to_str()
                .unwrap()
                .strip_prefix("https://teams.microsoft.com/v2#code=")
                .unwrap()
                .split("&")
                .next()
                .unwrap()
                .to_string())
        } else {
            Err("No Location header found.".to_string())
        }
    } else {
        Err(format!(
            "Request failed with status code: {} and at url: {}.",
            res.status(),
            login_url
        ))
    }
}

pub fn authorize_with_ests_persistant_token(
    ests_auth_persistant_token: String,
) -> Result<AuthorizationCode, String> {
    let challenge = gen_code_challenge();

    let reprocess_info = get_reprocess_url(challenge.clone(), ests_auth_persistant_token)?;

    if reprocess_info.light == "+" {
        return Err(
            "ESTS light token is empty: Did you choose to stay logged in when authenticating? Try clearing your cache to login again."
                .to_string(),
        );
    }

    let login_url = format!(
        "{}&sessionid={}",
        reprocess_info.url_login,
        reprocess_info.light.strip_prefix("+").unwrap()
    );

    let code = get_authorization_code_from_redirect(login_url, reprocess_info.persistent)?;
    Ok(AuthorizationCode {
        code,
        code_verifier: challenge,
    })
}

pub fn authorize_with_webview() -> Result<(AuthorizationCode, Vec<Cookie>), String> {
    let (server, server_name) = IpcOneShotServer::<AuthorizationInfo>::new().unwrap();

    let default_path = if cfg!(windows) {
        ".\\target\\debug\\auth_webview.exe"
    } else {
        "./target/debug/auth_webview"
    };

    let auth_webview_path =
        env::var("AUTH_WEBVIEW_PATH").unwrap_or_else(|_| default_path.to_string());

    let _child = Command::new(auth_webview_path)
        .stdin(Stdio::piped())
        .args(&[server_name])
        .stdout(Stdio::piped())
        .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1") // Linux fix
        .spawn()
        .expect("Failed to spawn client process (did you build auth_webview?)");

    let (_, authorization_info) = server.accept().unwrap();

    if !authorization_info.success {
        return Err("Failed to get authorization codes.".to_string());
    }

    let cookies = authorization_info.cookies.unwrap();

    Ok((authorization_info.authorization_codes.unwrap(), cookies))
}

pub fn get_or_gen_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    scope: String,
) -> AccessToken {
    let refresh_token = access_tokens
        .write()
        .unwrap()
        .entry("refresh_token".to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                let ests_persistant_token = get_cache::<Vec<Cookie>>("cookies.json")
                    .unwrap()
                    .iter()
                    .find(|s| s.name == "ESTSAUTHPERSISTENT")
                    .unwrap()
                    .value
                    .clone();

                let auth_code =
                    authorize_with_ests_persistant_token(ests_persistant_token).unwrap();
                *token =
                    gen_refresh_token_from_code(auth_code.code, auth_code.code_verifier).unwrap()
            }
        })
        .or_insert_with(|| {
            let ests_persistant_token = get_cache::<Vec<Cookie>>("cookies.json")
                .unwrap()
                .iter()
                .find(|s| s.name == "ESTSAUTHPERSISTENT")
                .unwrap()
                .value
                .clone();

            let auth_code = authorize_with_ests_persistant_token(ests_persistant_token).unwrap();
            gen_refresh_token_from_code(auth_code.code, auth_code.code_verifier).unwrap()
        })
        .clone();

    access_tokens
        .write()
        .unwrap()
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
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    access_token: AccessToken,
) -> AccessToken {
    access_tokens
        .write()
        .unwrap()
        .entry("skype_token".to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                *token = gen_skype_token(access_token.clone()).unwrap();
            }
        })
        .or_insert_with(|| gen_skype_token(access_token).unwrap())
        .clone()
}
