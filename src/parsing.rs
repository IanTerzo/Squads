use crate::components::cached_image::c_cached_gif;
use crate::components::cached_image::c_cached_image;
use crate::style;
use crate::Message;
use ahash::AHasher;
use base64::decode;
use directories::ProjectDirs;
use iced::border;
use iced::mouse;
use iced::padding;
use iced::widget::mouse_area;
use iced::widget::{
    column, container, rich_text, row, text, text::Span, Column, Container, Row, Space,
};
use iced::{font, Element, Font};
use image::image_dimensions;
use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::parser::inline::InlineRule;
use markdown_it::parser::inline::InlineState;
use markdown_it::plugins::extra::linkify;
use markdown_it::plugins::extra::strikethrough;
use markdown_it::plugins::extra::tables;
use markdown_it::{plugins, MarkdownIt, Node, NodeValue, Renderer};
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use unicode_properties::UnicodeEmoji;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug)]
pub struct BlockQuote(String);

impl NodeValue for BlockQuote {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("blockquote", &(vec![]));
        fmt.text(&self.0);
        fmt.close("blockquote");
    }
}

#[derive(Debug)]
pub struct BlockReply(String, String, Option<String>); // Content, Name, Id

impl NodeValue for BlockReply {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let id = self.2.clone().unwrap_or("0".to_string());

        let mut attrs_blockquote = node.attrs.clone();
        attrs_blockquote.push(("itemtype", "http://schema.skype.com/Reply".into()));
        attrs_blockquote.push(("itemid", id.clone().into()));

        let mut attrs_strong = node.attrs.clone();
        attrs_strong.push(("itemprop", "mri".into()));
        attrs_strong.push(("itemid", id.clone().into()));

        let mut attrs_span = node.attrs.clone();
        attrs_span.push(("itemprop", "time".into()));
        attrs_span.push(("itemid", id.clone().into()));

        let mut attrs_p = node.attrs.clone();
        attrs_p.push(("itemprop", "preview".into()));

        fmt.open("blockquote", &attrs_blockquote);
        fmt.open("strong", &attrs_strong);
        fmt.text(&self.1);
        fmt.close("strong");
        fmt.open("span", &attrs_span);
        fmt.close("span");
        fmt.open("p", &attrs_p);
        fmt.text(&self.0);
        fmt.close("p");

        fmt.close("blockquote");
    }
}

struct BlockQuoteScanner;

impl BlockRule for BlockQuoteScanner {
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let line = state.get_line(state.line).trim();
        if !line.starts_with(r#">"#) {
            return None;
        }

        let content = &line[1..line.len()];

        let cleaned = content.trim_start();

        if cleaned.starts_with('[') {
            if let Some(end) = cleaned.find(']') {
                let reply_name = cleaned[1..end].trim().to_string();
                let other_part = &cleaned[end + 1..cleaned.len()].trim();

                if other_part.starts_with('[') {
                    if let Some(end) = other_part.find(']') {
                        let reply_id = other_part[1..end].trim().to_string();
                        let other_part = &other_part[end + 1..other_part.len()].trim();
                        Some((
                            Node::new(BlockReply(
                                other_part.to_string(),
                                reply_name,
                                Some(reply_id),
                            )),
                            1,
                        ))
                    } else {
                        Some((
                            Node::new(BlockReply(other_part.to_string(), reply_name, None)),
                            1,
                        ))
                    }
                } else {
                    Some((
                        Node::new(BlockReply(other_part.to_string(), reply_name, None)),
                        1,
                    ))
                }
            } else {
                Some((Node::new(BlockQuote(cleaned.to_string())), 1))
            }
        } else {
            Some((Node::new(BlockQuote(cleaned.to_string())), 1))
        }
    }
}

#[derive(Debug)]
// This is a structure that represents your custom Node in AST.
pub struct NewlineParser;

// This defines how your custom node should be rendered.
impl NodeValue for NewlineParser {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.self_close("br", &(vec![] as Vec<(&str, String)>));
    }
}

// This is an extension for the inline subparser.
struct InlineNewlineScanner;

impl InlineRule for InlineNewlineScanner {
    const MARKER: char = '\n';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        println!("inline input: {}", input);

        if !input.starts_with("\n") {
            return None;
        }

        Some((Node::new(NewlineParser), 1))
    }
}

pub fn parse_message_markdown(text: String) -> String {
    // TODO: handle newlines

    // Replace multiple newlines with repeated <br>
    let mut processed = String::new();

    for line in text.lines() {
        if line.is_empty() {
            processed.push_str("<p>&nbsp;</p>");
        } else {
            processed.push_str(&format!("{}\n", line));
        }
    }

    println!("{}", processed);
    let mut md = MarkdownIt::new();
    md.block.add_rule::<BlockQuoteScanner>();
    md.inline.add_rule::<InlineNewlineScanner>();
    plugins::cmark::add(&mut md);
    plugins::html::add(&mut md);
    linkify::add(&mut md);
    strikethrough::add(&mut md);
    tables::add(&mut md);

    let html = md.parse(&processed).render();

    html
}

enum DynamicContainer {
    Row(Row<'static, Message>),
    RowWrapping(Container<'static, Message>), // I must put it in a container since iced_widget:row:Wrapping is not public
    Column(Column<'static, Message>),
}

impl DynamicContainer {
    fn push(self, element: Element<'static, Message>) -> Self {
        match self {
            DynamicContainer::Row(mut row) => {
                row = row.push(element);
                DynamicContainer::Row(row)
            }
            DynamicContainer::RowWrapping(row_container) => {
                DynamicContainer::RowWrapping(row_container)
            }
            DynamicContainer::Column(mut column) => {
                column = column.push(element);
                DynamicContainer::Column(column)
            }
        }
    }

    fn into_element(self) -> Element<'static, Message> {
        match self {
            DynamicContainer::Row(row) => row.into(),
            DynamicContainer::RowWrapping(row_container) => row_container.into(),
            DynamicContainer::Column(column) => column.into(),
        }
    }

    fn wrap(self) -> Self {
        match self {
            DynamicContainer::Row(row) => {
                let wrapped = row.wrap(); // `row.wrap()` moves `row`, so capture the result
                DynamicContainer::RowWrapping(container(wrapped))
            }
            DynamicContainer::RowWrapping(row_container) => {
                DynamicContainer::RowWrapping(row_container)
            }

            DynamicContainer::Column(column) => DynamicContainer::Column(column),
        }
    }
}

fn transform_html<'a>(
    theme: &'a style::Theme,
    element: scraper::ElementRef<'a>,
    mut cascading_properties: HashMap<&'a str, String>,
) -> DynamicContainer {
    let element_tagname = element.value().name();

    // Initialize container as either row or column based on the tag name
    let mut dynamic_container = if element_tagname == "body" {
        DynamicContainer::Column(column![])
    } else if element_tagname == "p" {
        DynamicContainer::Row(row![])
    } else {
        // Default to a row if no specific tag matches
        DynamicContainer::Row(row![])
    };

    if matches!(element_tagname, "strong" | "u" | "s" | "i" | "em") {
        cascading_properties.insert(element_tagname, element_tagname.to_string());
    } else if element_tagname == "span" {
        if let Some(attr) = element.attr("itemtype") {
            if attr == "http://schema.skype.com/Mention" {
                cascading_properties.insert("span", attr.to_string());
            }
        }
    } else if element_tagname == "a" {
        if let Some(attr) = element.attr("href") {
            cascading_properties.insert("a", attr.to_string());
        }
    }

    for child in element.children() {
        if let Some(child_element) = scraper::ElementRef::wrap(child) {
            let element_name = child_element.value().name();
            if child.has_children() && element_name != "blockquote" && element_name != "code" {
                dynamic_container = dynamic_container.push(
                    transform_html(theme, child_element, cascading_properties.clone())
                        .into_element(),
                );
            }
            // Special cases
            else if element_name == "br" {
                dynamic_container = dynamic_container.push(Space::new(10000, 0).into());
            } else if element_name == "img" {
                if let Some(itemtype) = child_element.attr("itemtype") {
                    if itemtype == "http://schema.skype.com/Emoji" {
                        if let Some(alt) = child_element.attr("alt") {
                            let font = Font::with_name("Twemoji");
                            dynamic_container = dynamic_container
                                .push(rich_text![Span::new(alt.to_string()).font(font)].into());
                        }
                    } else if itemtype == "http://schema.skype.com/AMSImage" {
                        // most consistent way to get the image id
                        let image_url = child_element.attr("src").unwrap().to_string();
                        let identifier = image_url
                            .replace("https:", "")
                            .replace("/", "")
                            .replace(":", ""); // Windows

                        let mut image_width = 400.0;
                        let mut image_height = 400.0;

                        let mut image_path = ProjectDirs::from("", "ianterzo", "squads")
                            .unwrap()
                            .cache_dir()
                            .to_path_buf();
                        image_path.push("image-cache");
                        image_path.push(format!("{}.jpeg", &identifier));

                        // Use the html sizes if not able to fetch the image dimensions or if the image has not been downloaded yet
                        match image_dimensions(&image_path) {
                            Ok((width, height)) => {
                                image_width = width as f32;
                                image_height = height as f32;
                            }
                            Err(e) => {
                                if let Some(width) = child_element.attr("width") {
                                    let width = width.parse().unwrap();
                                    image_width = width;
                                }

                                if let Some(height) = child_element.attr("height") {
                                    let height = height.parse().unwrap();
                                    image_height = height;
                                }
                            }
                        }

                        // Limit image sizes

                        if image_width == image_height && image_width > 280.0 {
                            image_width = 250.0;
                            image_height = 250.0;
                        }

                        if image_width >= 420.0 {
                            let factor = 400.0 / image_width;
                            image_width = 400.0;
                            image_height = image_height * factor;
                        }

                        if image_height >= 380.0 {
                            let factor = 400.0 / image_height;
                            image_height = 400.0;
                            image_width = image_width * factor;
                        }

                        let team_picture = mouse_area(c_cached_image(
                            identifier.clone(),
                            Message::AuthorizeImage(image_url, identifier.clone()),
                            image_width,
                            image_height,
                        ))
                        .on_release(Message::ExpandImage(identifier, "jpeg".to_string()))
                        .interaction(mouse::Interaction::Pointer);

                        dynamic_container = dynamic_container.push(team_picture.into());
                    } else if itemtype == "http://schema.skype.com/Giphy" {
                        if let Some(image_url) = child_element.attr("src") {
                            let identifier = xxh3_64(image_url.to_string().as_bytes()).to_string();

                            let mut image_width = 250.0;
                            let mut image_height = 250.0;

                            if let Some(width) = child_element.attr("width") {
                                let width = width.parse().unwrap();
                                image_width = width;
                            }

                            if let Some(height) = child_element.attr("height") {
                                let height = height.parse().unwrap();
                                image_height = height;
                            }
                            let team_picture = mouse_area(c_cached_gif(
                                identifier.clone(),
                                Message::DownloadImage(image_url.to_string(), identifier.clone()),
                                image_width,
                                image_height,
                            ))
                            .on_release(Message::ExpandImage(identifier, "gif".to_string()))
                            .interaction(mouse::Interaction::Pointer);
                            dynamic_container = dynamic_container.push(team_picture.into());
                        } else {
                            dynamic_container =
                                dynamic_container.push(text!("Failed to load gif").into());
                        }
                    }
                }
            } else if element_name == "blockquote" {
                let color = theme.colors.primary3;

                if let Some(itemtype) = child_element.attr("itemtype") {
                    if itemtype == "http://schema.skype.com/Reply" {
                        let mut name = "Unknown User".to_string();

                        if let Some(element) = child_element
                            .select(&Selector::parse("strong").unwrap())
                            .next()
                        {
                            name = element.text().collect::<String>();
                        }

                        if let Some(element) =
                            child_element.select(&Selector::parse("p").unwrap()).next()
                        {
                            let text = element.text().collect::<String>();

                            dynamic_container = dynamic_container.push(
                                container(
                                    container(column![
                                        text!("{}", name).color(theme.colors.demo_text).size(14),
                                        text!("{}", text)
                                    ])
                                    .padding(5)
                                    .style(move |_| {
                                        container::Style {
                                            background: Some(color.into()),
                                            border: border::rounded(5),
                                            ..Default::default()
                                        }
                                    }),
                                )
                                .padding(padding::bottom(8))
                                .into(),
                            );
                        }
                    }
                } else {
                    dynamic_container = dynamic_container.push(
                        container(row![
                            text("| ").color(theme.colors.demo_text),
                            transform_html(theme, child_element, cascading_properties.clone())
                                .into_element()
                        ])
                        .padding(5)
                        .style(move |_| container::Style {
                            background: Some(color.into()),
                            border: border::rounded(4),
                            ..Default::default()
                        })
                        .into(),
                    );
                }
            } else if element_name == "code" {
                let color = theme.colors.primary3;
                let mut raw_code = child_element.inner_html();

                raw_code = raw_code.replace("<br>", "\n");

                let lines: Vec<Element<Message>> = raw_code
                    .lines()
                    .map(|line| text!("{}", line.to_string()).into())
                    .collect();

                dynamic_container = dynamic_container.push(
                    container(column(lines))
                        .padding(5)
                        .style(move |_| container::Style {
                            background: Some(color.into()),
                            border: border::rounded(5),
                            ..Default::default()
                        })
                        .into(),
                );
            }
        } else if child.value().is_text() {
            // ID might be useful for hover logic
            //let mut state = AHasher::default();
            //let id = child.id().hash(&mut state);
            //println!("{:#?}", state.finish());
            let text_content = child.value().as_text().unwrap().text.to_string();
            // Remove things like newlines since it is handled separately
            let text_content = text_content.replace("\n", "").replace("\r", "");

            let words = text_content.split_inclusive(" ");

            let mut font_text = Font::default();
            let mut font_emojis = Font::with_name("Twemoji");
            let mut color = theme.colors.text;
            let mut underline = false;
            let mut strikethrough = false;
            let mut link_href: Option<String> = None;

            if let Some(property) = cascading_properties.get("strong") {
                // check for consistency
                if property == "strong" {
                    font_text.weight = font::Weight::Bold;
                    font_emojis.weight = font::Weight::Bold;
                }
            }
            if let Some(property) = cascading_properties.get("u") {
                if property == "u" {
                    underline = true;
                }
            }
            if let Some(property) = cascading_properties.get("s") {
                if property == "s" {
                    strikethrough = true;
                }
            }
            if let Some(property) = cascading_properties.get("i") {
                if property == "i" {
                    font_text.style = font::Style::Italic;
                    font_emojis.style = font::Style::Italic;
                }
            }
            if let Some(property) = cascading_properties.get("em") {
                if property == "em" {
                    font_text.style = font::Style::Italic;
                    font_emojis.style = font::Style::Italic;
                }
            }
            if let Some(value) = cascading_properties.get("a") {
                link_href = Some(value.clone());
                color = theme.colors.text_link;
            }
            if let Some(property) = cascading_properties.get("span") {
                if property == "http://schema.skype.com/Mention" {
                    color = theme.colors.text_link;
                }
            }

            // Turn every word into its own rich text (not ideal but necessary to mantain correct wrapping)

            for word in words {
                let mut spans = vec![];

                // Use a different font for emojis
                for char in word.chars() {
                    if char.is_emoji_char() && !char.is_digit(10) {
                        let mut text_span = Span::new(char.to_string());
                        text_span = text_span
                            .font(font_emojis)
                            .color(color)
                            .underline(underline)
                            .strikethrough(strikethrough);

                        spans.push(text_span);
                    } else {
                        let mut text_span = Span::new(char.to_string());
                        text_span = text_span
                            .font(font_text)
                            .color(color)
                            .underline(underline)
                            .strikethrough(strikethrough);

                        spans.push(text_span);
                    }
                }

                // TODO: show an underline when hovering a link

                if let Some(ref link) = link_href {
                    dynamic_container = dynamic_container.push(
                        mouse_area(rich_text(spans))
                            .on_release(Message::LinkClicked(link.clone()))
                            .interaction(mouse::Interaction::Pointer)
                            .into(),
                    );
                } else {
                    dynamic_container = dynamic_container.push(rich_text(spans).into());
                };
            }
        }
    }

    dynamic_container.wrap() // If it is a row, it needs to be wrapping
}

pub fn parse_message_html<'a>(
    theme: &style::Theme,
    content: String,
) -> Result<Element<'a, Message>, String> {
    let content = content.trim_end().trim_end_matches("<p>&nbsp;</p>"); //Remove tailing space
    let document = Html::parse_document(content);

    let selector = Selector::parse("body").unwrap();
    if let Some(root_element) = document.select(&selector).next() {
        Ok(transform_html(theme, root_element, HashMap::new()).into_element())
    } else {
        Err("Couldn't get body from message html".to_string())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCard {
    pub attachments: Vec<Attachment>,
    #[serde(rename = "type")]
    pub media_card_type: String,
    pub entities: Vec<Value>, // Not sure
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub content: Content,
    pub content_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub body: Vec<Value>, // Vec of items
    pub actions: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MsTeams {
    pub width: String,
}

pub fn parse_card_html<'a>(content: String) -> Result<Element<'a, Message>, String> {
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("Swift").unwrap();

    if let Some(swift_element) = document.select(&selector).next() {
        let b64_value = swift_element.value().attr("b64").unwrap();
        let decoded_bytes = decode(b64_value).unwrap();
        let decoded_string = std::str::from_utf8(&decoded_bytes).unwrap();
        //let parsed: MediaCard = serde_json::from_str(decoded_string).unwrap();
        //println!("{parsed:#?}");
        Ok(text!("{decoded_string}").into())
    } else {
        Err("Couldn't find Swift tag from card HTML".to_string())
    }
}

pub fn parse_content_emojis<'a>(content: String) -> Element<'a, Message> {
    let mut text_row = row![];

    for char in content.chars() {
        if char.is_emoji_char() && !char.is_digit(10) {
            text_row = text_row.push(text(char).font(Font::with_name("Twemoji")));
        } else {
            text_row = text_row.push(text(char));
        }
    }

    text_row.into()
}

pub fn get_html_preview(html: &str) -> String {
    let document = Html::parse_document(html);

    document
        .root_element()
        .text() // Iterator over all text nodes
        .map(|s| s.trim()) // Remove extra whitespace
        .filter(|s| !s.is_empty()) // Skip empty text
        .collect::<Vec<_>>()
        .join(" ") // Join with space
}
