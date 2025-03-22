use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{fs, io::Write};

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
