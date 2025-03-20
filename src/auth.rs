use crate::api::{gen_refresh_token_from_code, gen_skype_token, gen_tokens, AccessToken};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};

fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizationCode {
    pub code: String,
    pub code_verifier: String,
}

use ipc_channel::ipc::IpcOneShotServer;

pub fn authorize() -> Result<AuthorizationCode, String> {
    let (server, server_name) = IpcOneShotServer::<AuthorizationCode>::new().unwrap();

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

    let (_, authorization_codes) = server.accept().unwrap();

    if authorization_codes.code == "" && authorization_codes.code_verifier == "" {
        return Err("Failed to get authorization codes.".to_string());
    }

    Ok(authorization_codes)
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
