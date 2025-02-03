use crate::widgets::viewport::ViewportHandler;
use iced::widget::{
    column, container, image, rich_text, row, text, text::Span, Column, Container, Row, Space,
};
use iced::{font, Color, ContentFit, Element, Font};
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

fn transform_html(
    element: scraper::ElementRef,
    mut cascading_properties: Vec<String>,
) -> DynamicContainer {
    let element_tagname = element.value().name();

    // Initialize container as either row or column based on the tag name
    let mut container = if element_tagname == "body" || element_tagname == "div" {
        DynamicContainer::Column(column![])
    } else if element_tagname == "p" {
        DynamicContainer::Row(row![])
    } else {
        // Default to a row if no specific tag matches
        DynamicContainer::Row(row![])
    };

    if matches!(element_tagname, "strong" | "u" | "s") {
        cascading_properties.push(element_tagname.to_string());
    }

    for child in element.children() {
        if child.has_children() {
            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                container = container.push(
                    transform_html(child_element, cascading_properties.clone()).into_element(),
                );
            }
        } else if child.value().is_text() {
            let text_content = child.value().as_text().unwrap().text.to_string();

            let words = text_content.split_inclusive(" ");

            // Turn every word into its own rich text (not ideal but necessary to mantain correct wrapping)
            for word in words {
                let mut text_span = Span::new(word.to_string());

                for property in &cascading_properties {
                    if property == "strong" {
                        let font_bold = Font {
                            weight: font::Weight::Bold,
                            ..Default::default()
                        };
                        text_span = text_span.font(font_bold);
                    }
                    if property == "s" {
                        text_span = text_span.strikethrough(true);
                    }
                    if property == "u" {
                        text_span = text_span.underline(true);
                    }
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
    let content = content.replace("\n", "").replace("\r", "");
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("body").unwrap();
    if let Some(root_element) = document.select(&selector).next() {
        transform_html(root_element, vec![]).into_element()
    } else {
        text("").into()
    }
}

pub fn message<'a>(message: crate::api::Message) -> Element<'a, Message> {
    let mut message_column = column![].padding(20).spacing(20);

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
    }
    message_column.into()
}
