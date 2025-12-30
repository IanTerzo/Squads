use iced::widget::text_editor::Content;
use iced::widget::{
    column, container, mouse_area, rich_text, row, space, span, svg, text, text_editor, text_input,
    tooltip,
};
use iced::{Alignment, Border, Element, Font, Length, Padding, border, font, padding};
use indexmap::IndexMap;

use crate::components::emoji_picker::c_emoji_picker;
use crate::components::toooltip::c_tooltip;
use crate::types::{Emoji, MessageAreaAction};
use crate::widgets::anchored_overlay::anchored_overlay;
use crate::{Message, Page};
use crate::{style, utils};

pub fn c_message_area<'a>(
    theme: &'a style::Theme,
    message_area_content: &'a Content,
    subject_input_content: &Option<String>,
    page: Page,
    message_area_height: &f32,
    show_emoji_picker: &bool,
    search_emojis_input_value: &String,
    emoji_map: &'a IndexMap<String, Emoji>,
    window_size: &(f32, f32),
) -> Element<'a, Message> {
    container(
        container(
            column![
                if let Some(subject_input_content) = subject_input_content {
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
                            }),
                    )
                    .padding(padding::vertical(2))
                } else {
                    container(space())
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
                        row![
                            anchored_overlay(
                                tooltip(
                                    mouse_area(
                                        svg(utils::get_image_dir().join("smile.svg"))
                                            .width(19)
                                            .height(19)
                                    )
                                    .on_release(Message::ToggleMessageAreaEmojiPicker)
                                    .interaction(iced::mouse::Interaction::Pointer),
                                    c_tooltip(theme, "Emojis"),
                                    tooltip::Position::Top
                                ),
                                c_emoji_picker(
                                    theme,
                                    search_emojis_input_value,
                                    emoji_map,
                                    |emoji_id, emoji_unicode| Message::EmojiPickerSend(
                                        emoji_id,
                                        emoji_unicode
                                    )
                                ),
                                crate::widgets::anchored_overlay::Position::Top,
                                *message_area_height + 28.0,
                                *show_emoji_picker,
                                *window_size
                            ),
                            tooltip(
                                mouse_area(
                                    svg(utils::get_image_dir().join("upload.svg"))
                                        .width(19)
                                        .height(19)
                                )
                                .on_release(Message::UploadFile)
                                .interaction(iced::mouse::Interaction::Pointer),
                                c_tooltip(theme, "Upload File"),
                                tooltip::Position::Top
                            )
                        ]
                        .spacing(8),
                        container(
                            row![
                                row![
                                    tooltip(
                                        mouse_area(
                                            rich_text![span::<(), Font>("B").font(Font {
                                                weight: font::Weight::Bold,
                                                ..Default::default()
                                            })]
                                            .size(20)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Bold
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Bold"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            rich_text![span::<(), Font>("I").font(Font {
                                                style: font::Style::Italic,
                                                ..Default::default()
                                            })]
                                            .size(20)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Italic
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Italic"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            rich_text![span::<(), Font>("U").underline(true)]
                                                .size(20)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Underline
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Underline"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            rich_text![span::<(), Font>("S").strikethrough(true)]
                                                .size(20)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Striketrough
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Strikethrough"),
                                        tooltip::Position::Top
                                    )
                                ]
                                .spacing(8),
                                row![
                                    tooltip(
                                        mouse_area(
                                            svg(utils::get_image_dir().join("list.svg"))
                                                .width(23)
                                                .height(23)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::List
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Unordered List"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            svg(utils::get_image_dir().join("list-ordered.svg"))
                                                .width(23)
                                                .height(23)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::OrderedList
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Ordered List"),
                                        tooltip::Position::Top
                                    )
                                ]
                                .padding(padding::top(3))
                                .spacing(8),
                                row![
                                    tooltip(
                                        mouse_area(
                                            svg(utils::get_image_dir().join("code.svg"))
                                                .width(23)
                                                .height(23)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Code
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Code"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            svg(utils::get_image_dir().join("text-quote.svg"))
                                                .width(23)
                                                .height(23)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Blockquote
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Blockquote"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            svg(utils::get_image_dir().join("link.svg"))
                                                .width(19)
                                                .height(19)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Link
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Link"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        mouse_area(
                                            svg(utils::get_image_dir().join("image.svg"))
                                                .width(19)
                                                .height(19)
                                        )
                                        .on_release(Message::MessageAreaAction(
                                            MessageAreaAction::Image
                                        ))
                                        .interaction(iced::mouse::Interaction::Pointer),
                                        c_tooltip(theme, "Image"),
                                        tooltip::Position::Top
                                    ),
                                    tooltip(
                                        svg(utils::get_image_dir().join("at-sign.svg"))
                                            .width(19)
                                            .height(19),
                                        c_tooltip(theme, "Mention"),
                                        tooltip::Position::Top
                                    )
                                ]
                                .padding(padding::top(3))
                                .spacing(8),
                            ]
                            .spacing(18)
                        )
                    ]
                    .align_y(Alignment::Center)
                    .spacing(28),
                    container(row![
                        if let Page::Team(_, _) = page {
                            if subject_input_content.is_none() {
                                container(
                                    mouse_area(
                                        container(
                                            row![
                                                svg(utils::get_image_dir().join("plus.svg"))
                                                    .width(19)
                                                    .height(19),
                                                text("Add subject")
                                            ]
                                            .align_y(Alignment::Center)
                                            .spacing(6),
                                        )
                                        .padding(4),
                                    )
                                    .on_release(Message::AddSubject)
                                    .interaction(iced::mouse::Interaction::Pointer),
                                )
                            } else {
                                container(
                                    mouse_area(
                                        container(
                                            row![
                                                svg(utils::get_image_dir().join("minus.svg"))
                                                    .width(19)
                                                    .height(19),
                                                text("Remove Subject")
                                                    .color(theme.colors.demo_text)
                                            ]
                                            .align_y(Alignment::Center)
                                            .spacing(6),
                                        )
                                        .padding(4),
                                    )
                                    .on_release(Message::RemoveSubject)
                                    .interaction(iced::mouse::Interaction::Pointer),
                                )
                            }
                        } else {
                            container(space())
                        },
                        if let Page::Team(_, _) = page {
                            space().width(12)
                        } else {
                            space()
                        },
                        container(
                            mouse_area(
                                container(
                                    row![
                                        svg(utils::get_image_dir().join("corner-down-right.svg"))
                                            .width(19)
                                            .height(19),
                                        text("Enter to send")
                                    ]
                                    .align_y(Alignment::Center)
                                    .spacing(6)
                                )
                                .padding(4)
                            )
                            .on_release(Message::PostMessage)
                            .interaction(iced::mouse::Interaction::Pointer)
                        ),
                    ])
                    .align_right(Length::Fill)
                ]
                .align_y(Alignment::Center)
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
