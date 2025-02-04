use crate::widgets::viewport::ViewportHandler;
use iced::widget::{
    column, container, image, rich_text, row, text, text::Span, Column, Container, Row, Space,
};
use iced::{alignment, font, Color, ContentFit, Element, Font};
use std::collections::HashMap;
use std::path::Path;

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
    let mut container = if element_tagname == "body" {
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
                container = container.push(
                    transform_html(child_element, cascading_properties.clone()).into_element(),
                );
            }
            // Special case
            else if child_element.value().name() == "br" {
                container = container.push(Space::new(10000, 0).into());
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
                container = container.push(text.into());
            }
        }
    }

    container.wrap() // If it is a row, it needs to be wrapping
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

pub fn c_message<'a>(message: crate::api::Message) -> Option<Element<'a, Message>> {
    let mut message_column = column![].spacing(20);

    if !message.properties.systemdelete {
        if message.message_type == "RichText/Html" {
            let im_display_name = message.im_display_name.unwrap();
            let user_id = message.from.unwrap();

            let mut user_picture =
                container(ViewportHandler::new(Space::new(0, 0)).on_enter_unique(
                    user_id.clone(),
                    Message::FetchUserImage(user_id.clone(), im_display_name.clone()),
                ))
                .style(|_| container::Style {
                    background: Some(
                        Color::parse("#b8b4b4")
                            .expect("Background color is invalid.")
                            .into(),
                    ),

                    ..Default::default()
                })
                .height(31)
                .width(31);

            let image_path = format!("image-cache/user-{}.jpeg", user_id.clone());

            if Path::new(&image_path).exists() {
                user_picture = container(
                    ViewportHandler::new(
                        image(image_path)
                            .content_fit(ContentFit::Cover)
                            .width(31)
                            .height(31),
                    )
                    .on_enter_unique(
                        user_id.clone(),
                        Message::FetchUserImage(user_id.clone(), im_display_name.clone()),
                    ),
                )
                .height(31)
                .width(31)
            }
            let message_info = row![user_picture, text!("{}", im_display_name)]
                .spacing(10)
                .wrap();

            message_column = message_column.push(message_info);
        }

        if message.properties.subject != "".to_string() {
            message_column =
                message_column.push(text(message.properties.subject).size(18).font(font::Font {
                    weight: font::Weight::Bold,
                    ..Default::default()
                }));
        }
        if message.message_type == "RichText/Html" {
            if let Some(content) = message.content {
                message_column = message_column.push(parse_message_html(content));
            }
        } else {
            if let Some(content) = message.content {
                message_column = message_column.push(text(content));
            }
        }

        return Some(message_column.into());
    }

    None
}
