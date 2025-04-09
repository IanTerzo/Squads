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

pub async fn get_or_gen_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    scope: String,
    tenant: &String,
) -> AccessToken {
    let refresh_token = {
        let tokens = access_tokens.read().unwrap();
        tokens.get("refresh_token").unwrap().clone()
    };

    let refresh_token = if refresh_token.expires < get_epoch_s() {
        let new_token = renew_refresh_token(&refresh_token, tenant.clone())
            .await
            .unwrap();

        {
            let mut tokens = access_tokens.write().unwrap();
            tokens.insert("refresh_token".to_string(), new_token.clone());
        }

        new_token
    } else {
        refresh_token
    };

    let maybe_token = {
        let tokens = access_tokens.read().unwrap();
        tokens.get(&scope).cloned()
    };

    if let Some(token) = maybe_token {
        if token.expires >= get_epoch_s() {
            return token;
        }
    }

    let new_token = gen_token(&refresh_token, scope.clone(), tenant.clone())
        .await
        .unwrap();

    {
        let mut tokens = access_tokens.write().unwrap();
        tokens.insert(scope.clone(), new_token.clone());
    }

    save_to_cache("access_tokens.json", &*access_tokens.read().unwrap());

    new_token
}

pub async fn get_or_gen_skype_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    access_token: AccessToken,
) -> AccessToken {
    let maybe_token = {
        let tokens = access_tokens.read().unwrap();
        tokens.get("skype_token").cloned()
    };
    if let Some(token) = maybe_token {
        if token.expires >= get_epoch_s() {
            return token;
        }
    }
    let new_token = gen_skype_token(&access_token).await.unwrap();

    {
        let mut tokens = access_tokens.write().unwrap();
        tokens.insert("skype_token".to_string(), new_token.clone());
    }

    save_to_cache("access_tokens.json", &*access_tokens.read().unwrap());

    new_token
}
