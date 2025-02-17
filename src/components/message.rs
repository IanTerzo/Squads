use iced::widget::{
    column, container, rich_text, row, text, text::Span, Column, Container, Row, Space,
};
use iced::{border, font, Alignment, Color, Element, Font};
use std::collections::HashMap;
use std::fmt::format;

use crate::components::cached_image::c_cached_image;
use crate::widgets::viewport::ViewportHandler;
use crate::Message;

use scraper::{Html, Selector};

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

    if matches!(element_tagname, "strong" | "u" | "s") {
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
                let itemtype = child_element.attr("itemtype").unwrap();

                if itemtype == "http://schema.skype.com/Emoji" {
                    if let Some(alt) = child_element.attr("alt") {
                        dynamic_container =
                            dynamic_container.push(rich_text![Span::new(alt.to_string())].into());
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

                    println!("width: {}", image_width);
                    println!("height: {}", image_height);

                    let team_picture = c_cached_image(
                        image_id.clone(),
                        Message::AuthorizeImage(image_id.clone()),
                        image_width,
                        image_height,
                    );

                    dynamic_container = dynamic_container.push(team_picture.into());
                }
            }
        } else if child.value().is_text() {
            let text_content = child.value().as_text().unwrap().text.to_string();

            let words = text_content.split_inclusive(" ");

            let mut font = Font {
                ..Default::default()
            };
            let mut color = Color::from_rgb(1.0, 1.0, 1.0);
            let mut underline = false;
            let mut strikethrough = false;
            let mut link_href: Option<String> = None;

            if let Some(property) = cascading_properties.get("strong") {
                // check for consistency
                if property == "strong" {
                    font = Font {
                        weight: font::Weight::Bold,
                        ..Default::default()
                    };
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

fn parse_message_html(content: String) -> Element<'static, Message> {
    // Remove things like newlines to avoid them being treated as text during the parsing
    // replace <br> as newlines (hotfix?)
    let content = content.replace("\n", "").replace("\r", "");
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("body").unwrap();
    if let Some(root_element) = document.select(&selector).next() {
        transform_html(root_element, HashMap::new()).into_element()
    } else {
        text("").into()
    }
}

pub fn c_message<'a>(
    message: crate::api::Message,
    emoji_map: &HashMap<String, String>,
) -> Option<Element<'a, Message>> {
    let mut message_column = column![].spacing(20);

    if let Some(properties) = message.properties.clone() {
        if properties.systemdelete {
            return None;
        }
    }

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" {
            let im_display_name = message.im_display_name.unwrap();
            let user_id = message.from.unwrap();
            let user_picture = c_cached_image(
                user_id.clone(),
                Message::FetchUserImage(user_id.clone(), im_display_name.clone()),
                31.0,
                31.0,
            );

            message_info = message_info.push(user_picture);
            message_info = message_info.push(text!("{}", im_display_name));
        }
    }

    if let Some(arrival_time) = message.original_arrival_time {
        let parsed_time: Vec<&str> = arrival_time.split("T").collect();
        let date = parsed_time[0].replace("-", "/");
        let time_chunks: Vec<&str> = parsed_time[1].split(":").collect();
        let time = format!("{}:{}", time_chunks[0], time_chunks[1]);

        println!("{time}");

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

    if let Some(properties) = message.properties.clone() {
        if properties.subject != "".to_string() {
            message_column = message_column.push(
                text(message.properties.clone().unwrap().subject)
                    .size(18)
                    .font(font::Font {
                        weight: font::Weight::Bold,
                        ..Default::default()
                    }),
            );
        }
    }

    if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" {
            if let Some(content) = message.content {
                message_column = message_column.push(parse_message_html(content));
            }
        }
        //else if message_type == "text"
        else {
            if let Some(content) = message.content {
                message_column = message_column.push(text(content));
            }
        }
    }

    if let Some(properties) = message.properties {
        if let Some(reactions) = properties.emotions {
            let mut reactions_row = row![].spacing(10);
            for reaction in reactions {
                let reacters = reaction.users.len();
                if reacters == 0 {
                    continue;
                }
                let mut reaction_char = "?".to_string();

                let reaction_val = emoji_map.get(&reaction.key);
                if let Some(reaction_unicode) = reaction_val {
                    reaction_char = reaction_unicode.clone();
                }

                let reaction_val = format!("{} {}", reaction_char, reacters);
                let reaction_container = container(rich_text![Span::new(reaction_val)])
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
                reactions_row = reactions_row.push(reaction_container);
            }
            message_column = message_column.push(reactions_row);
        }
    }

    return Some(message_column.into());
}
