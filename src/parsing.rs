use crate::components::cached_image::c_cached_image;
use crate::style;
use crate::Message;
use base64::decode;
use directories::ProjectDirs;
use iced::widget::{
    column, container, rich_text, row, text, text::Span, Column, Container, Row, Space,
};
use iced::{font, Element, Font};
use image::image_dimensions;
use markdown_it::{plugins, MarkdownIt};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;

pub fn parse_message_markdown(text: String) -> String {
    let mut md = MarkdownIt::new();
    plugins::cmark::add(&mut md);
    let html = md.parse(text.as_str()).render();
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
    theme: &style::Theme,
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
            if child.has_children() {
                dynamic_container = dynamic_container.push(
                    transform_html(theme, child_element, cascading_properties.clone())
                        .into_element(),
                );
            }
            // Special cases
            else if child_element.value().name() == "br" {
                dynamic_container = dynamic_container.push(Space::new(10000, 0).into());
            } else if child_element.value().name() == "img" {
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

                        let team_picture = c_cached_image(
                            identifier.clone(),
                            Message::AuthorizeImage(image_url, identifier),
                            image_width,
                            image_height,
                        );

                        dynamic_container = dynamic_container.push(team_picture.into());
                    }
                }
            }
        } else if child.value().is_text() {
            let text_content = child.value().as_text().unwrap().text.to_string();

            let words = text_content.split_inclusive(" ");

            let mut font = Font::default();
            let mut color = theme.colors.text;
            let mut underline = false;
            let mut strikethrough = false;
            let mut link_href: Option<String> = None;

            if let Some(property) = cascading_properties.get("strong") {
                // check for consistency
                if property == "strong" {
                    font.weight = font::Weight::Bold;
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
                    font.style = font::Style::Italic;
                }
            }
            if let Some(property) = cascading_properties.get("em") {
                if property == "em" {
                    font.style = font::Style::Italic;
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
                let mut text_span = Span::new(word.to_string());
                text_span = text_span
                    .font(font)
                    .color(color)
                    .underline(underline)
                    .strikethrough(strikethrough);

                if let Some(ref link) = link_href {
                    text_span = text_span.link(Message::LinkClicked(link.clone()));
                }

                // Wrap in rich_text and push to container
                let text = rich_text![text_span];
                dynamic_container = dynamic_container.push(text.into());
            }
        }
    }

    dynamic_container.wrap() // If it is a row, it needs to be wrapping
}

pub fn parse_message_html<'a>(
    theme: &style::Theme,
    content: String,
) -> Result<Element<'a, Message>, String> {
    // Remove things like newlines to avoid them being treated as text during the parsing
    let content = content.replace("\n", "").replace("\r", "");
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("body").unwrap();
    if let Some(root_element) = document.select(&selector).next() {
        Ok(transform_html(theme, root_element, HashMap::new()).into_element())
    } else {
        Err("Couldn't get body from message html".to_string())
    }
}

use serde::Serialize;
use serde_json::Value;

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
        let parsed: MediaCard = serde_json::from_str(decoded_string).unwrap();
        //println!("{parsed:#?}");
        Ok(text!("{decoded_string}").into())
    } else {
        Err("Couldn't find Swift tag from card HTML".to_string())
    }
}
