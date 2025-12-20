use iced::widget::text_editor::Content;
use iced::widget::{
    column, container, mouse_area, rich_text, row, span, svg, text, text_editor, text_input,
};
use iced::{border, font, padding, Alignment, Border, Element, Font, Length, Padding};

use crate::components::horizontal_line::c_horizontal_line;
use crate::types::{EmojiPickerAction, EmojiPickerLocation, MessageAreaAction};
use crate::Message;
use crate::{style, utils};

pub fn c_message_area<'a>(
    theme: &'a style::Theme,
    message_area_content: &'a Content,
    subject_input_content: Option<&String>,
    message_area_height: &f32,
) -> Element<'a, Message> {
    container(
        container(
            column![
                if let Some(subject_input_content) = subject_input_content {
                    column![
                        container(
                            text_input("Subject", &subject_input_content)
                                .font(Font {
                                    weight: font::Weight::Bold,
                                    ..Default::default()
                                })
                                .on_input(Message::SubjectInputContentChanged)
                                .padding(6)
                                .style(|_, _| text_input::Style {
                                    background: theme.colors.foreground.into(),
                                    border: border::rounded(6),
                                    icon: theme.colors.not_set,
                                    placeholder: theme.colors.demo_text,
                                    value: theme.colors.text,
                                    selection: theme.colors.text_selection,
                                },)
                        )
                        .padding(2),
                        container(c_horizontal_line(&theme, Length::Fill)).padding(Padding {
                            left: 6.0,
                            right: 6.0,
                            top: 0.0,
                            bottom: 6.0
                        })
                    ]
                } else {
                    column![]
                },
                container(
                    text_editor(message_area_content)
                        .height(*message_area_height)
                        .on_action(move |action| Message::MessageAreaEdit(action))
                        .placeholder("Type your message...")
                        .style(|_, _| text_editor::Style {
                            background: theme.colors.foreground.into(),
                            border: border::rounded(4),
                            placeholder: theme.colors.demo_text,
                            value: theme.colors.text,
                            selection: theme.colors.text_selection,
                        })
                        .id("message_area")
                ),
                row![
                    row![
                        mouse_area(
                            svg(utils::get_image_dir().join("smile.svg"))
                                .width(20)
                                .height(20)
                        )
                        .on_release(Message::ToggleEmojiPicker(
                            Some(EmojiPickerLocation::OverMessageArea),
                            EmojiPickerAction::Send
                        ))
                        .interaction(iced::mouse::Interaction::Pointer),
                        mouse_area(
                            svg(utils::get_image_dir().join("upload.svg"))
                                .width(20)
                                .height(20)
                        )
                        .on_release(Message::UploadFile)
                        .interaction(iced::mouse::Interaction::Pointer),
                    ]
                    .spacing(8),
                    container(
                        row![
                            row![
                                mouse_area(
                                    rich_text![span::<(), Font>("B").font(Font {
                                        weight: font::Weight::Bold,
                                        ..Default::default()
                                    })]
                                    .size(20)
                                )
                                .on_release(Message::MessageAreaAction(MessageAreaAction::Bold))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    rich_text![span::<(), Font>("I").font(Font {
                                        style: font::Style::Italic,
                                        ..Default::default()
                                    })]
                                    .size(20)
                                )
                                .on_release(Message::MessageAreaAction(MessageAreaAction::Italic))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    rich_text![span::<(), Font>("U").underline(true)].size(20)
                                )
                                .on_release(Message::MessageAreaAction(
                                    MessageAreaAction::Underline
                                ))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    rich_text![span::<(), Font>("S").strikethrough(true)].size(20)
                                )
                                .on_release(Message::MessageAreaAction(
                                    MessageAreaAction::Striketrough
                                ))
                                .interaction(iced::mouse::Interaction::Pointer),
                            ]
                            .spacing(8),
                            row![
                                mouse_area(
                                    svg(utils::get_image_dir().join("list.svg"))
                                        .width(23)
                                        .height(23)
                                )
                                .on_release(Message::MessageAreaAction(MessageAreaAction::List))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    svg(utils::get_image_dir().join("list-ordered.svg"))
                                        .width(23)
                                        .height(23)
                                )
                                .on_release(Message::MessageAreaAction(
                                    MessageAreaAction::OrderedList
                                ))
                                .interaction(iced::mouse::Interaction::Pointer),
                            ]
                            .padding(padding::top(3))
                            .spacing(8),
                            row![
                                mouse_area(
                                    svg(utils::get_image_dir().join("code.svg"))
                                        .width(23)
                                        .height(23)
                                )
                                .on_release(Message::MessageAreaAction(MessageAreaAction::Code))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    svg(utils::get_image_dir().join("text-quote.svg"))
                                        .width(23)
                                        .height(23)
                                )
                                .on_release(Message::MessageAreaAction(
                                    MessageAreaAction::Blockquote
                                ))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    svg(utils::get_image_dir().join("link.svg"))
                                        .width(19)
                                        .height(19)
                                )
                                .on_release(Message::MessageAreaAction(MessageAreaAction::Link))
                                .interaction(iced::mouse::Interaction::Pointer),
                                mouse_area(
                                    svg(utils::get_image_dir().join("image.svg"))
                                        .width(19)
                                        .height(19)
                                )
                                .on_release(Message::MessageAreaAction(MessageAreaAction::Image))
                                .interaction(iced::mouse::Interaction::Pointer),
                                svg(utils::get_image_dir().join("at-sign.svg"))
                                    .width(19)
                                    .height(19)
                            ]
                            .padding(padding::top(3))
                            .spacing(8),
                        ]
                        .spacing(20)
                    ),
                    container(
                        row![
                            container(
                                mouse_area(
                                    container(
                                        row![
                                            svg(utils::get_image_dir().join("plus.svg"))
                                                .width(19)
                                                .height(19),
                                            text("Add subject")
                                        ]
                                        .spacing(6)
                                    )
                                    .padding(4)
                                    .align_y(Alignment::Center)
                                )
                                .on_release(Message::PostMessage)
                                .interaction(iced::mouse::Interaction::Pointer)
                            ),
                            container(
                                mouse_area(
                                    container(
                                        row![
                                            svg(utils::get_image_dir()
                                                .join("corner-down-right.svg"))
                                            .width(19)
                                            .height(19),
                                            text("Enter to send")
                                        ]
                                        .spacing(6)
                                    )
                                    .padding(4)
                                    .align_y(Alignment::Center)
                                )
                                .on_release(Message::PostMessage)
                                .interaction(iced::mouse::Interaction::Pointer)
                            ),
                        ]
                        .spacing(12)
                    )
                    .align_right(Length::Fill)
                ]
            ]
            .padding(Padding {
                top: 18.0,
                right: 20.0,
                bottom: 18.0,
                left: 20.0,
            }),
        )
        .style(|_| container::Style {
            background: Some(theme.colors.foreground.into()),
            border: Border {
                color: theme.colors.line,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
    )
    .into()
}
