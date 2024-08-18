use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub value: String,
    pub expires: u64,
}

type SharedStore<T> = Arc<Mutex<T>>;

pub struct Store {
    persistent_store_path: PathBuf,
    tokens_storage: SharedStore<HashMap<String, AccessToken>>,
    cache_storage: SharedStore<HashMap<String, Value>>,
}

impl Store {
    fn global_tokens_storage() -> SharedStore<HashMap<String, AccessToken>> {
        static mut TOKENS_STORAGE: Option<SharedStore<HashMap<String, AccessToken>>> = None;
        static INIT: std::sync::Once = std::sync::Once::new();

        unsafe {
            INIT.call_once(|| {
                TOKENS_STORAGE = Some(Arc::new(Mutex::new(HashMap::new())));
            });
            TOKENS_STORAGE.clone().unwrap()
        }
    }

    fn global_cache_storage() -> SharedStore<HashMap<String, Value>> {
        static mut CACHE_STORAGE: Option<SharedStore<HashMap<String, Value>>> = None;
        static INIT: std::sync::Once = std::sync::Once::new();

        unsafe {
            INIT.call_once(|| {
                CACHE_STORAGE = Some(Arc::new(Mutex::new(HashMap::new())));
            });
            CACHE_STORAGE.clone().unwrap()
        }
    }

    pub fn new(persistent_store_path: PathBuf) -> Self {
        Store {
            persistent_store_path,
            tokens_storage: Store::global_tokens_storage(),
            cache_storage: Store::global_cache_storage(),
        }
    }

    pub fn get_token(&self, scope: &str) -> Option<AccessToken> {
        self.tokens_storage.lock().get(scope).cloned()
    }

    pub fn set_token(&self, scope: &str, token: AccessToken) {
        self.tokens_storage.lock().insert(scope.to_string(), token);
    }

    pub fn set_data(&self, key: &str, value: Value) {
        self.cache_storage.lock().insert(key.to_string(), value);
    }

    pub fn get_data(&self, key: &str) -> Option<Value> {
        self.cache_storage.lock().get(key).cloned()
    }
}
