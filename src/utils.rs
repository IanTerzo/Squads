use directories::ProjectDirs;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::{
    env, fs,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn truncate_name(name: String, max_length: usize) -> String {
    if name.len() > max_length {
        let cutoff = max_length.saturating_sub(3);
        let mut end = name.len();

        for (idx, _) in name.char_indices() {
            if idx > cutoff {
                end = idx;
                break;
            }
        }

        let mut truncated = name[..end].to_string();
        truncated.push_str("...");
        truncated
    } else {
        name.to_string()
    }
}

pub fn save_to_cache<T>(filename: &str, content: &T)
where
    T: Serialize,
{
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");
    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    fs::create_dir_all(cache_dir.clone()).expect("Failed to create cache directory");

    cache_dir.push(filename);

    let json = serde_json::to_string_pretty(content).expect("Failed to serialize content");
    let mut file = fs::File::create(cache_dir).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn get_cache<T: DeserializeOwned>(filename: &str) -> Option<T> {
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    cache_dir.push(filename);

    if cache_dir.exists() {
        let file_content = fs::read_to_string(cache_dir).ok()?;
        serde_json::from_str(&file_content).ok()
    } else {
        None
    }
}

pub fn delete_cache(filename: &str) {
    if let Some(project_dirs) = ProjectDirs::from("", "ianterzo", "squads") {
        let cache_path = project_dirs.cache_dir().join(filename);
        let _ = fs::remove_file(cache_path);
    }
}

pub fn get_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn get_image_dir() -> PathBuf {
    PathBuf::from(env::var("SQUADS_IMAGE_DIR").unwrap_or("images".to_string()))
}

pub fn get_resource_dir() -> PathBuf {
    PathBuf::from(env::var("SQUADS_RESOURCE_DIR").unwrap_or("resources".to_string()))
}

pub fn get_local_ip() -> String {
    // Try to determine local IP by connecting a UDP socket to a public address.
    // This doesn't send any data, just lets the OS pick the outbound interface.
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}
