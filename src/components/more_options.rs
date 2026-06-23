use crate::api::Profile;
use crate::components::horizontal_line::c_horizontal_line;
use crate::parsing::get_html_preview;
use crate::utils;
use crate::{Message, style};
use iced::alignment::Vertical;
use crate::widgets::click_area::click_area;
use iced::widget::{column, container, row, space, svg, text};
use iced::{Border, Element};

pub fn c_more_options<'a>(
    theme: &'a style::Theme,
    message: crate::api::Message,
    me: &Profile,
) -> Element<'a, Message> {
    click_area(
        container(column![
            column![
                click_area(
                    row![
                        svg(utils::get_image_dir().join("smile.svg"))
                            .width(19)
                            .height(19),
                        text("Add reaction")
                    ]
                    .align_y(Vertical::Center)
                    .spacing(8)
                )
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(Message::ToggleMessageEmojiPicker(
                    message.id.clone().unwrap()
                )),
                c_horizontal_line(theme, 200.into()),
                click_area(
                    row![
                        svg(utils::get_image_dir().join("reply.svg"))
                            .width(19)
                            .height(19),
                        text("Reply")
                    ]
                    .align_y(Vertical::Center)
                    .spacing(8)
                )
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(Message::Reply(
                    message.content.clone(),
                    message.im_display_name.clone(),
                    message.id.clone(),
                )),
                click_area(
                    row![
                        svg(utils::get_image_dir().join("copy.svg"))
                            .width(19)
                            .height(19),
                        text("Copy Text")
                    ]
                    .align_y(Vertical::Center)
                    .spacing(8)
                )
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(Message::CopyText(
                    if let Some(content) = message.content.clone() {
                        if let Some(message_type) = message.message_type.clone() {
                            if message_type == "RichText/Html" {
                                get_html_preview(&content)
                            } else {
                                content
                            }
                        } else {
                            content
                        }
                    } else {
                        "".to_string()
                    }
                )),
            ]
            .spacing(12),
            if message.from.clone().unwrap_or("none".to_string()) == format!("8:orgid:{}", me.id) {
                column![
                    space(),
                    c_horizontal_line(theme, 200.into()),
                    row![
                        svg(utils::get_image_dir().join("pencil.svg"))
                            .width(19)
                            .height(19),
                        text("Edit message")
                    ]
                    .align_y(Vertical::Center)
                    .spacing(8),
                    if let Some(properties) = message.properties
                        && properties.deletetime > 0
                    {
                        click_area(
                            row![
                                svg(utils::get_image_dir().join("rotate-ccw.svg"))
                                    .width(19)
                                    .height(19),
                                text("Restore Message")
                            ]
                            .align_y(Vertical::Center)
                            .spacing(8),
                        )
                        .interaction(iced::mouse::Interaction::Pointer)
                        .on_press(Message::Restore(message.id.clone()))
                    } else {
                        click_area(
                            row![
                                svg(utils::get_image_dir().join("trash.svg"))
                                    .width(19)
                                    .height(19),
                                text("Delete Message")
                            ]
                            .align_y(Vertical::Center)
                            .spacing(8),
                        )
                        .interaction(iced::mouse::Interaction::Pointer)
                        .on_press(Message::Delete(message.id.clone()))
                    }
                ]
                .spacing(12)
            } else {
                column![]
            }
        ])
        .padding(15)
        .style(|_| container::Style {
            background: Some(theme.colors.tooltip.into()),
            border: Border {
                color: theme.colors.line,
                width: 1.0,
                radius: 4.into(),
            },
            ..Default::default()
        }),
    )
    .on_enter(Message::EnterMoreOptions)
    .on_exit(Message::ExitMoreOptions)
    .into()
}
