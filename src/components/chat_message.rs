use crate::Message;
use crate::api::Profile;
use crate::components::cached_image::c_cached_image;
use crate::components::emoji_picker::c_emoji_picker;
use crate::components::horizontal_line::c_horizontal_line;
use crate::components::picture_and_status::c_picture_and_status;
use crate::components::toooltip::c_tooltip;
use crate::parsing::{get_html_preview, parse_card_html, parse_message_html};
use crate::style;
use crate::types::Emoji;
use crate::utils;
use crate::websockets::Presence;
use crate::widgets::anchored_overlay::anchored_overlay;
use crate::widgets::selectable_text;
use crate::widgets::selectable_text::selectable_text;
use iced::alignment::Vertical;
use iced::widget::tooltip::Position;
use iced::widget::{column, container, mouse_area, row, space, stack, svg, text, tooltip};
use iced::{Alignment, Border, Element, Font, Length, Padding, border, font, padding};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::time::Duration;

const LOG_THREAD_ACTIVITY: bool = false;

pub fn c_chat_message<'a>(
    theme: &'a style::Theme,
    message: crate::api::Message,
    chat_thread_id: &'a String,
    chat_message_options: &HashMap<String, bool>,
    emoji_map: &'a IndexMap<String, Emoji>,
    users: &HashMap<String, Profile>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
    show_more_options: &'a bool,
    more_menu_message_id: &'a Option<String>,
    show_emoji_picker: &'a bool,
    emoji_picker_message_id: &'a Option<String>,
    search_emojis_input_value: &String,
) -> Option<Element<'a, Message>> {
    if let Some(message_type) = message.message_type.clone() {
        if message_type.contains("ThreadActivity") && !LOG_THREAD_ACTIVITY {
            return None;
        }
    }

    let mut message_row = row![].spacing(6);

    let mut contents_column = column![].spacing(4);

    // Message info bar

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" || message_type == "Text" {
            if let Some(ref user_id) = message.from {
                let display_name =
                    if let Some(profile) = users.get(&user_id.replace("8:orgid:", "")) {
                        profile.display_name.clone().unwrap_or(
                            message
                                .im_display_name
                                .clone()
                                .unwrap_or("Unknown User".to_string()),
                        )
                    } else {
                        message
                            .im_display_name
                            .clone()
                            .unwrap_or("Unknown User".to_string())
                    };

                let presence = user_presences.get(user_id);

                let identifier = user_id.clone().replace(":", "");

                let user_picture = c_cached_image(
                    identifier.clone(),
                    Message::FetchUserImage(identifier, user_id.clone(), display_name.clone()),
                    31.0,
                    31.0,
                    4.0,
                );

                message_row = message_row.push(c_picture_and_status(
                    theme,
                    user_picture,
                    presence,
                    (31.0, 31.0),
                ));
                message_info = message_info.push(text!("{}", display_name).font(Font {
                    weight: font::Weight::Bold,
                    ..Default::default()
                }));
            } else {
                if let Some(display_name) = message.im_display_name.clone() {
                    message_info = message_info.push(text(display_name));
                } else {
                    message_info = message_info.push(text("Unknown User"));
                }
            }
        } else if message_type == "RichText/Media_Card" {
            if let Some(display_name) = message.im_display_name.clone() {
                message_info = message_info.push(text(display_name));
            } else {
                message_info = message_info.push(text("Unknown User"));
            }
        }
    }

    if let Some(arrival_time) = message.original_arrival_time {
        let parsed_time: Vec<&str> = arrival_time.split("T").collect();
        let date = parsed_time[0].replace("-", "/");
        let time_chunks: Vec<&str> = parsed_time[1].split(":").collect();
        let time = format!("{}:{}", time_chunks[0], time_chunks[1]);

        message_info = message_info.push(text(date).size(14).color(theme.colors.demo_text));
        message_info = message_info.push(text(time).size(14).color(theme.colors.demo_text));
    }

    contents_column = contents_column.push(message_info.wrap());

    // Message content

    let deleted = if let Some(properties) = &message.properties {
        properties.deletetime != 0 || properties.systemdelete
    } else {
        false
    };

    if deleted {
        contents_column = contents_column.push(text("Message deleted").font(Font {
            style: font::Style::Italic,
            ..Font::default()
        }));
    } else if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" {
            if let Some(content) = message.content.clone() {
                match parse_message_html(theme, content) {
                    Ok(result) => {
                        contents_column = contents_column.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        } else if message_type == "RichText/Media_Card" {
            if let Some(content) = message.content.clone() {
                match parse_card_html(content) {
                    Ok(result) => {
                        contents_column = contents_column.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        } else if message_type == "Text" {
            if let Some(content) = message.content.clone() {
                contents_column = contents_column.push(selectable_text(content).style(|_| {
                    selectable_text::Style {
                        color: None,
                        selection_color: theme.colors.text_selection,
                    }
                }));
            }
        } else {
            if let Some(content) = message.content.clone() {
                contents_column = contents_column.push(selectable_text(content).style(|_| {
                    selectable_text::Style {
                        color: None,
                        selection_color: theme.colors.text_selection,
                    }
                }));
            }
        }
    }

    // Files

    if !deleted {
        if let Some(properties) = &message.properties {
            if let Some(files) = &properties.files {
                let mut files_row = row![].spacing(10);

                for file in files {
                    let file_container = mouse_area(
                        container(
                            row![
                                svg(utils::get_image_dir().join("paperclip.svg"))
                                    .width(16)
                                    .height(16),
                                text(file.file_name.clone().unwrap_or("File".to_string()))
                                    .color(theme.colors.text_link)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(8),
                        )
                        .style(|_| container::Style {
                            background: Some(theme.colors.foreground_surface.into()),
                            border: border::rounded(6),
                            ..Default::default()
                        })
                        .padding(12),
                    )
                    .on_release(Message::DownloadFile(file.clone()))
                    .interaction(iced::mouse::Interaction::Pointer);
                    files_row = files_row.push(file_container);
                }

                contents_column = contents_column.push(files_row.wrap());
            }
        }
    }

    // Message reactions

    let mut reactions_row = row![].spacing(8);

    let mut are_reactions = false;

    if !deleted {
        if let Some(properties) = &message.properties {
            if let Some(reactions) = &properties.emotions {
                for reaction in reactions {
                    let reacters = reaction.users.len();
                    if reacters == 0 {
                        continue;
                    }

                    let mut reaction_text = text("(?)");

                    let reaction_val = emoji_map.get(&reaction.key);
                    if let Some(reaction_val) = reaction_val {
                        reaction_text = text(reaction_val.unicode.clone());
                    }

                    let mut self_has_reacted = false;
                    for reactor in &reaction.users {
                        if let Some(reactor_id) = reactor.mri.split(":").nth(2) {
                            if reactor_id == me.id {
                                self_has_reacted = true
                            }
                        }
                    }

                    let reaction_string = reaction
                        .users
                        .iter()
                        .filter_map(|reactor| {
                            if let Some(reactor_id) = reactor.mri.split(":").nth(2) {
                                users.get(reactor_id).map(|profile| {
                                    profile
                                        .display_name
                                        .clone()
                                        .unwrap_or("Unknown User".to_string())
                                })
                            } else {
                                Some("Unknown User".to_string())
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(", ");

                    let reaction_container = tooltip(
                        mouse_area(
                            container(row![reaction_text, text(reacters)].spacing(4))
                                .style(move |_| {
                                    if self_has_reacted {
                                        container::Style {
                                            background: Some(theme.colors.emotion_selected.into()),
                                            border: Border {
                                                color: theme.colors.emotion_selected_line,
                                                width: 1.0,
                                                radius: 4.0.into(),
                                            },
                                            ..Default::default()
                                        }
                                    } else {
                                        container::Style {
                                            background: Some(
                                                theme.colors.foreground_surface.into(),
                                            ),
                                            border: border::rounded(4),
                                            ..Default::default()
                                        }
                                    }
                                })
                                .padding(Padding {
                                    top: 3.0,
                                    right: 3.0,
                                    bottom: 3.0,
                                    left: 5.0,
                                })
                                .align_y(Alignment::Center),
                        )
                        .on_release(Message::EmotionClicked(
                            message.id.clone().unwrap(),
                            reaction.clone(),
                        ))
                        .interaction(iced::mouse::Interaction::Pointer),
                        container(text(format!(
                            "{} \"{}\" reacted by {}",
                            reaction_val
                                .map(|emoji| emoji.unicode.clone())
                                .unwrap_or("(?)".to_string()),
                            reaction.key.clone(),
                            reaction_string
                        )))
                        .padding(12)
                        .max_width(200)
                        .style(|_| container::Style {
                            background: Some(theme.colors.foreground_surface.into()),
                            border: border::rounded(4),
                            ..Default::default()
                        }),
                        tooltip::Position::Top,
                    )
                    .delay(Duration::from_millis(400));

                    reactions_row = reactions_row.push(reaction_container);

                    are_reactions = true;
                }
            }
        }
    }

    if are_reactions {
        reactions_row = reactions_row.push(tooltip(
            mouse_area(
                container(
                    svg(utils::get_image_dir().join("plus.svg"))
                        .width(19)
                        .height(19),
                )
                .align_y(Vertical::Center)
                .style(|_| container::Style {
                    background: Some(theme.colors.foreground_surface.into()),
                    border: border::rounded(4),
                    ..Default::default()
                })
                .padding(Padding {
                    top: 4.0,
                    right: 5.0,
                    bottom: 4.0,
                    left: 5.0,
                }),
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::ToggleMessageEmojiPicker(
                message.id.clone().unwrap(),
            )),
            c_tooltip(theme, "Add Reaction"),
            Position::Top,
        ));

        contents_column = contents_column.push(reactions_row.wrap());
    }

    message_row = message_row.push(container(contents_column).width(Length::Fill));

    // Actions container

    let mut action_container = container(row![]);

    // Fill the container if the message is being hovered.

    let is_hovered = chat_message_options
        .get(&message.id.clone().unwrap())
        .unwrap_or(&false)
        .to_owned();

    let message_id_clone = message.id.clone().unwrap();
    if (is_hovered && more_menu_message_id.is_none() && emoji_picker_message_id.is_none())
        || *more_menu_message_id == message.id.clone()
        || *emoji_picker_message_id == message.id.clone()
    {
        action_container =
            container(
                container(
                    container(
                        row![
                            if message.from.clone().unwrap_or("none".to_string())
                                == format!("8:orgid:{}", me.id)
                            {
                                container(row![
                                    tooltip(
                                        svg(utils::get_image_dir().join("pencil.svg"))
                                            .width(17)
                                            .height(17),
                                        c_tooltip(theme, "Edit"),
                                        tooltip::Position::Top
                                    )
                                    .gap(3),
                                    space().width(6),
                                ])
                            } else {
                                container(space())
                            },
                            tooltip(
                                mouse_area(
                                    svg(utils::get_image_dir().join("reply.svg"))
                                        .width(21)
                                        .height(21)
                                )
                                .on_release(Message::Reply(
                                    message.content.clone(),
                                    message.im_display_name.clone(),
                                    message.id.clone(),
                                )),
                                c_tooltip(theme, "Reply"),
                                tooltip::Position::Top
                            )
                            .gap(3),
                            space().width(6),
                            anchored_overlay(
                                tooltip(
                                    mouse_area(
                                        svg(utils::get_image_dir().join("plus.svg"))
                                            .width(21)
                                            .height(21)
                                    )
                                    .on_release(
                                        Message::ToggleMessageEmojiPicker(
                                            message.id.clone().unwrap()
                                        )
                                    ),
                                    c_tooltip(theme, "React"),
                                    tooltip::Position::Top
                                )
                                .gap(3),
                                c_emoji_picker(
                                    theme,
                                    search_emojis_input_value,
                                    emoji_map,
                                    move |emoji_id, emoji_unicode| Message::EmojiPickerReaction(
                                        emoji_id,
                                        emoji_unicode,
                                        message_id_clone.clone(),
                                        chat_thread_id.clone()
                                    )
                                ),
                                crate::widgets::anchored_overlay::Position::Left,
                                5.0,
                                *show_emoji_picker
                            ),
                            space().width(6),
                            anchored_overlay(
                                tooltip(
                                    mouse_area(
                                        svg(utils::get_image_dir().join("ellipsis.svg"))
                                            .width(21)
                                            .height(21)
                                    )
                                    .on_release(
                                        Message::ToggleShowMoreOptions(message.id.clone().unwrap())
                                    ),
                                    c_tooltip(theme, "More"),
                                    tooltip::Position::Top
                                )
                                .gap(3),
                                mouse_area(
                                    container(
                                        column![
                                            row![
                                                svg(utils::get_image_dir().join("smile.svg"))
                                                    .width(19)
                                                    .height(19),
                                                text("Add reaction")
                                            ]
                                            .align_y(Vertical::Center)
                                            .spacing(8),
                                            c_horizontal_line(theme, 200.into()),
                                            row![
                                                svg(utils::get_image_dir().join("pencil.svg"))
                                                    .width(19)
                                                    .height(19),
                                                text("Edit message")
                                            ]
                                            .align_y(Vertical::Center)
                                            .spacing(8),
                                            mouse_area(
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
                                            .on_release(Message::Reply(
                                                message.content.clone(),
                                                message.im_display_name.clone(),
                                                message.id.clone(),
                                            )),
                                            mouse_area(
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
                                            .on_release(Message::CopyText(
                                                if let Some(content) = message.content.clone() {
                                                    if let Some(message_type) =
                                                        message.message_type.clone()
                                                    {
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
                                            c_horizontal_line(theme, 200.into()),
                                            row![
                                                svg(utils::get_image_dir().join("trash.svg"))
                                                    .width(19)
                                                    .height(19),
                                                text("Delete Message")
                                            ]
                                            .align_y(Vertical::Center)
                                            .spacing(8)
                                        ]
                                        .spacing(12)
                                    )
                                    .padding(15)
                                    .style(|_| {
                                        container::Style {
                                            background: Some(theme.colors.tooltip.into()),
                                            border: Border {
                                                color: theme.colors.line,
                                                width: 1.0,
                                                radius: 4.into(),
                                            },
                                            ..Default::default()
                                        }
                                    })
                                )
                                .on_enter(Message::EnterMoreOptions)
                                .on_exit(Message::ExitMoreOptions),
                                crate::widgets::anchored_overlay::Position::Left,
                                2.0,
                                *show_more_options
                            ),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding {
                        top: 3.0,
                        right: 6.0,
                        bottom: 3.0,
                        left: 6.0,
                    })
                    .style(|_| container::Style {
                        background: Some(theme.colors.foreground_surface.into()),
                        border: border::rounded(4),
                        ..Default::default()
                    }),
                )
                .padding(padding::right(10))
                .align_y(Alignment::Center),
            )
            .align_right(iced::Length::Fill);
    }

    let message_id_clone = message.id.clone();
    let message_stack = stack!(
        mouse_area(
            container(message_row)
                .style(move |_| container::Style {
                    background: if (is_hovered
                        && more_menu_message_id.is_none()
                        && emoji_picker_message_id.is_none())
                        || *more_menu_message_id == message_id_clone
                        || *emoji_picker_message_id == message_id_clone
                    {
                        Some(theme.colors.message_hovered.into())
                    } else {
                        Some(theme.colors.background.into())
                    },
                    border: border::rounded(4),
                    ..Default::default()
                })
                .padding(Padding {
                    top: 4.0,
                    right: 1.0,
                    bottom: 4.0,
                    left: 3.0,
                })
        )
        .on_enter(Message::ShowChatMessageOptions(message.id.clone().unwrap()))
        .on_exit(Message::StopShowChatMessageOptions(message.id.unwrap())),
        action_container
    );

    return Some(message_stack.into());
}
