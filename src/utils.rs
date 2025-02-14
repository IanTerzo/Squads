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
