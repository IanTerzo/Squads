use crate::api::{gen_skype_token, gen_token, renew_refresh_token, AccessToken};
use crate::utils::{get_epoch_s, save_to_cache};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};
extern crate reqwest;

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

pub fn get_or_gen_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    scope: String,
    tenant: &String,
) -> AccessToken {
    let mut refresh_token = access_tokens
        .write()
        .unwrap()
        .get(&"refresh_token".to_string())
        .unwrap()
        .clone();

    if refresh_token.expires < get_epoch_s() {
        refresh_token = renew_refresh_token(&refresh_token, tenant.clone()).unwrap();
    }

    let token = access_tokens
        .write()
        .unwrap()
        .entry(scope.to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                *token = gen_token(&refresh_token, scope.to_string(), tenant.clone()).unwrap();
            }
        })
        .or_insert_with(|| gen_token(&refresh_token, scope.to_string(), tenant.clone()).unwrap())
        .clone();

    save_to_cache("access_tokens.json", &*access_tokens.read().unwrap());

    token
}

pub fn get_or_gen_skype_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    access_token: AccessToken,
) -> AccessToken {
    let token = access_tokens
        .write()
        .unwrap()
        .entry("skype_token".to_string())
        .and_modify(|token| {
            if token.expires < get_epoch_s() {
                *token = gen_skype_token(&access_token).unwrap();
            }
        })
        .or_insert_with(|| gen_skype_token(&access_token).unwrap())
        .clone();

    save_to_cache("access_tokens.json", &*access_tokens.read().unwrap());

    token
}
