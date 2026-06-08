use crate::api::{AccessToken, gen_skype_token, gen_token, renew_refresh_token};
use crate::utils::{delete_cache, get_epoch_s, save_to_cache};
use iced::Task;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};
extern crate reqwest;

#[derive(Debug, Clone)]
pub enum AuthError {
    TokenExpired(String),
    Other(String),
}

fn classify_auth_error(err: &str) -> AuthError {
    if err.contains("AADSTS700082")
        || err.contains("AADSTS50173")
        || err.contains("AADSTS70000")
        || err.contains("AADSTS700084")
        || err.contains("invalid_grant")
    {
        AuthError::TokenExpired(err.to_string())
    } else {
        AuthError::Other(err.to_string())
    }
}

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

pub async fn get_or_gen_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    scope: &str,
    tenant: &str,
) -> Result<AccessToken, AuthError> {
    let refresh_token = {
        let tokens = access_tokens.read().unwrap();
        tokens
            .get("refresh_token")
            .cloned()
            .ok_or_else(|| AuthError::TokenExpired("No refresh token found".to_string()))?
    };

    let refresh_token = if refresh_token.expires < get_epoch_s() {
        match renew_refresh_token(&refresh_token, tenant).await {
            Ok(new_token) => {
                let mut tokens = access_tokens.write().unwrap();
                tokens.insert("refresh_token".to_string(), new_token.clone());
                new_token
            }
            Err(e) => {
                let auth_err = classify_auth_error(&e);
                if matches!(auth_err, AuthError::TokenExpired(_)) {
                    access_tokens.write().unwrap().remove("refresh_token");
                    delete_cache("access_tokens.json");
                }
                return Err(auth_err);
            }
        }
    } else {
        refresh_token
    };

    let maybe_token = {
        let tokens = access_tokens.read().unwrap();
        tokens.get(scope).cloned()
    };

    if let Some(token) = maybe_token {
        if token.expires >= get_epoch_s() {
            return Ok(token);
        }
    }

    match gen_token(&refresh_token, scope, tenant).await {
        Ok(new_token) => {
            {
                let mut tokens = access_tokens.write().unwrap();
                tokens.insert(scope.to_string(), new_token.clone());
            }
            save_to_cache("access_tokens.json", &*access_tokens.read().unwrap());
            Ok(new_token)
        }
        Err(e) => {
            let auth_err = classify_auth_error(&e);
            if matches!(auth_err, AuthError::TokenExpired(_)) {
                access_tokens.write().unwrap().remove("refresh_token");
                delete_cache("access_tokens.json");
            }
            Err(auth_err)
        }
    }
}

pub async fn get_or_gen_skype_token(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    access_token: AccessToken,
) -> Result<AccessToken, AuthError> {
    let maybe_token = {
        let tokens = access_tokens.read().unwrap();
        tokens.get("skype_token").cloned()
    };
    if let Some(token) = maybe_token {
        if token.expires >= get_epoch_s() {
            return Ok(token);
        }
    }
    let new_token = gen_skype_token(&access_token)
        .await
        .map_err(|e| AuthError::Other(e.to_string()))?;

    {
        let mut tokens = access_tokens.write().unwrap();
        tokens.insert("skype_token".to_string(), new_token.clone());
    }

    save_to_cache("access_tokens.json", &*access_tokens.read().unwrap());

    Ok(new_token)
}

pub fn authenticated_task<T, F, Fut, Msg>(
    access_tokens: Arc<RwLock<HashMap<String, AccessToken>>>,
    scope: &str,
    tenant: &str,
    action: F,
    callback: impl FnOnce(Result<T, AuthError>) -> Msg + Send + 'static,
) -> Task<Msg>
where
    F: FnOnce(AccessToken) -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send,
    T: Send + 'static,
    Msg: Send + 'static,
{
    let scope = scope.to_string();
    let tenant = tenant.to_string();
    Task::perform(
        async move {
            match get_or_gen_token(access_tokens, &scope, &tenant).await {
                Ok(token) => Ok(action(token).await),
                Err(e) => Err(e),
            }
        },
        callback,
    )
}
