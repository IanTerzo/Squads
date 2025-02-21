use iced::widget::{
    column, container, rich_text, row, text, text::Span, Column, Container, Row, Space,
};
use iced::{border, font, Alignment, Color, Element, Font, Padding};
use std::collections::HashMap;
use unicode_properties::UnicodeEmoji;

use crate::components::cached_image::c_cached_image;
use crate::Message;
use base64::decode;
use scraper::{Html, Selector};

const LOG_THREAD_ACTIVITY: bool = false;

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

    if matches!(element_tagname, "strong" | "u" | "s" | "i") {
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
                    transform_html(child_element, cascading_properties.clone()).into_element(),
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
                        let image_id = child_element
                            .attr("src")
                            .unwrap()
                            .replace("https://eu-api.asm.skype.com/v1/objects/", "")
                            .replace(
                                "https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/",
                                "",
                            )
                            .replace("/views/imgo", "");

                        let mut image_width = 20.0;
                        let mut image_height = 20.0;

                        if let Some(width) = child_element.attr("width") {
                            let width = width.parse().unwrap();
                            image_width = width;
                        }

                        if let Some(height) = child_element.attr("height") {
                            let height = height.parse().unwrap();
                            image_height = height;
                        }

                        let team_picture = c_cached_image(
                            image_id.clone(),
                            Message::AuthorizeImage(image_id.clone()),
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
            let mut color = Color::from_rgb(1.0, 1.0, 1.0);
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
            if let Some(value) = cascading_properties.get("a") {
                link_href = Some(value.clone());
                color = Color::from_rgb(0.4, 0.5961, 0.851);
            }
            if let Some(property) = cascading_properties.get("span") {
                if property == "http://schema.skype.com/Mention" {
                    color = Color::from_rgb(0.4, 0.5961, 0.851);
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

fn parse_message_html(content: String) -> Result<Element<'static, Message>, String> {
    // Remove things like newlines to avoid them being treated as text during the parsing
    let content = content.replace("\n", "").replace("\r", "");
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("body").unwrap();
    if let Some(root_element) = document.select(&selector).next() {
        Ok(transform_html(root_element, HashMap::new()).into_element())
    } else {
        Err("Couldn't get body from message html".to_string())
    }
}

fn parse_card_html<'a>(content: String) -> Result<Element<'a, Message>, String> {
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("Swift").unwrap();

    if let Some(swift_element) = document.select(&selector).next() {
        let b64_value = swift_element.value().attr("b64").unwrap();
        let decoded_bytes = decode(b64_value).unwrap();
        let decoded_string = std::str::from_utf8(&decoded_bytes).unwrap();

        Ok(text!("{decoded_string}").into())
    } else {
        Err("Couldn't find Swift tag from card HTML".to_string())
    }
}

pub fn c_message<'a>(
    message: crate::api::Message,
    emoji_map: &HashMap<String, String>,
) -> Option<Element<'a, Message>> {
    if let Some(message_type) = message.message_type.clone() {
        if message_type.contains("ThreadActivity") && !LOG_THREAD_ACTIVITY {
            return None;
        }
    }

    let mut message_column = column![].spacing(20);

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    // Message info bar

    if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" || message_type == "Text" {
            if let Some(display_name) = message.im_display_name {
                let user_id = message.from.unwrap();
                let user_picture = c_cached_image(
                    user_id.clone(),
                    Message::FetchUserImage(user_id.clone(), display_name.clone()),
                    31.0,
                    31.0,
                );

                message_info = message_info.push(user_picture);
                message_info = message_info.push(text!("{}", display_name));
            } else {
                message_info = message_info.push(text("Unknown User"));
            }
        } else if message_type == "RichText/Media_Card" {
            if let Some(display_name) = message.im_display_name {
                message_info = message_info.push(text!("{}", display_name));
            } else {
                message_info = message_info.push(text("Unknown"));
            }
        }
    }

    if let Some(arrival_time) = message.original_arrival_time {
        let parsed_time: Vec<&str> = arrival_time.split("T").collect();
        let date = parsed_time[0].replace("-", "/");
        let time_chunks: Vec<&str> = parsed_time[1].split(":").collect();
        let time = format!("{}:{}", time_chunks[0], time_chunks[1]);

        message_info = message_info.push(text(date).size(14).color(Color::from_rgb(
            0.788235294117647,
            0.788235294117647,
            0.788235294117647,
        )));
        message_info = message_info.push(text(time).size(14).color(Color::from_rgb(
            0.788235294117647,
            0.788235294117647,
            0.788235294117647,
        )));
    }

    message_column = message_column.push(message_info);

    // Message subject

    if let Some(properties) = message.properties.clone() {
        if properties.subject != "".to_string() {
            let mut text_row = row![];

            for c in message.properties.clone().unwrap().subject.chars() {
                if c.is_emoji_char() {
                    text_row = text_row.push(text(c).font(Font::with_name("Twemoji")).size(18));
                } else {
                    text_row = text_row.push(text(c).size(18).font(font::Font {
                        weight: font::Weight::Bold,
                        ..Default::default()
                    }));
                }
            }

            message_column = message_column.push(text_row);
        }
    }

    // Message content

    let deleted = if let Some(properties) = message.properties.clone() {
        properties.deletetime != 0 || properties.systemdelete
    } else {
        false
    };

    if deleted {
        message_column = message_column.push(text("Message deleted").font(Font {
            style: font::Style::Italic,
            ..Font::default()
        }));
    } else if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" {
            if let Some(content) = message.content {
                match parse_message_html(content) {
                    Ok(result) => {
                        message_column = message_column.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        } else if message_type == "RichText/Media_Card" {
            if let Some(content) = message.content {
                match parse_card_html(content) {
                    Ok(result) => {
                        message_column = message_column.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        } else if message_type == "Text" {
            if let Some(content) = message.content {
                let mut text_row = row![];

                for c in content.chars() {
                    if c.is_emoji_char() {
                        text_row = text_row.push(text(c).font(Font::with_name("Twemoji")));
                    } else {
                        text_row = text_row.push(text(c));
                    }
                }

                message_column = message_column.push(text_row);
            }
        } else {
            if let Some(content) = message.content {
                message_column = message_column.push(text(content));
            }
        }
    }

    // Message reactions

    if !deleted {
        let mut reactions_row = row![].spacing(10);

        if let Some(properties) = message.properties {
            if let Some(reactions) = properties.emotions {
                for reaction in reactions {
                    let reacters = reaction.users.len();
                    if reacters == 0 {
                        continue;
                    }
                    let mut reaction_text = text("(?)");
                    let font = Font::with_name("Twemoji");

                    let reaction_val = emoji_map.get(&reaction.key);
                    if let Some(reaction_unicode) = reaction_val {
                        reaction_text = text(reaction_unicode.clone()).font(font);
                    }

                    let reaction_container =
                        container(row![reaction_text, text(reacters)].spacing(4))
                            .style(|_| container::Style {
                                background: Some(
                                    Color::parse("#525252")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(4),
                                ..Default::default()
                            })
                            .padding(Padding {
                                top: 3.0,
                                right: 3.0,
                                bottom: 3.0,
                                left: 5.0,
                            })
                            .align_y(Alignment::Center);
                    reactions_row = reactions_row.push(reaction_container);
                }
            }
        }

        let add_reaction_container =
            container(row![text("+")].spacing(4).padding(Padding::from([0, 3])))
                .style(|_| container::Style {
                    background: Some(
                        Color::parse("#525252")
                            .expect("Background color is invalid.")
                            .into(),
                    ),
                    border: border::rounded(4),
                    ..Default::default()
                })
                .padding(3)
                .align_y(Alignment::Center);

        reactions_row = reactions_row.push(add_reaction_container);

        message_column = message_column.push(reactions_row);
    }

    return Some(message_column.into());
}
