use markdown_it::{plugins, MarkdownIt};

pub fn parse_message_markdown(text: String) -> String {
    let mut md = MarkdownIt::new();
    plugins::cmark::add(&mut md);
    let html = md.parse(text.as_str()).render();
    html
}
