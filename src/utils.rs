pub fn truncate_name(name: &str, max_length: usize) -> String {
    if name.len() > max_length {
        let mut truncated = name.to_string();
        truncated.replace_range(max_length - 3.., "...");
        truncated
    } else {
        name.to_string()
    }
}
