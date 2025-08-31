use crate::api::Profile;
use crate::components::cached_image::c_cached_image;
use crate::components::picture_and_status::c_picture_and_status;
use crate::parsing::{parse_card_html, parse_message_html};
use crate::style;
use crate::websockets::Presence;
use crate::widgets::circle::circle;
use crate::Message;
use iced::widget::{column, container, mouse_area, row, stack, svg, text, Space};
use iced::{border, font, padding, Alignment, Element, Font, Length, Padding};
use std::collections::HashMap;

const LOG_THREAD_ACTIVITY: bool = false;

pub fn c_chat_minimized_message<'a>(
    theme: &'a style::Theme,
    message: crate::api::Message,
    chat_message_options: &HashMap<String, bool>,
    emoji_map: &HashMap<String, String>,
    users: &HashMap<String, Profile>,
    user_presences: &'a HashMap<String, Presence>,
) -> Option<Element<'a, Message>> {
    if let Some(message_type) = message.message_type.clone() {
        if message_type.contains("ThreadActivity") && !LOG_THREAD_ACTIVITY {
            return None;
        }
    }

    let mut contents_column = column![].spacing(4);

    let mut message_row = row![].spacing(3).width(Length::Fill);

    let is_hovered = chat_message_options
        .get(&message.id.clone().unwrap())
        .unwrap_or(&false)
        .to_owned();

    // Time
    if is_hovered {
        if let Some(arrival_time) = message.original_arrival_time {
            let parsed_time: Vec<&str> = arrival_time.split("T").collect();
            let time_chunks: Vec<&str> = parsed_time[1].split(":").collect();
            let time = format!("{}:{}", time_chunks[0], time_chunks[1]);

            message_row = message_row.push(
                container(text(time).size(14).color(theme.colors.demo_text)).padding(Padding {
                    left: 3.0,
                    right: 5.0,
                    top: 2.0,
                    bottom: 0.0,
                }),
            )
        } else {
            message_row = message_row.push(Space::new(43, 1))
        }
    } else {
        message_row = message_row.push(Space::new(43, 1))
    }

    // Message content

    let deleted = if let Some(properties) = &message.properties {
        properties.deletetime != 0 || properties.systemdelete
    } else {
        false
    };

    if deleted {
        message_row = message_row.push(text("Message deleted").font(Font {
            style: font::Style::Italic,
            ..Font::default()
        }));
    } else if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" {
            if let Some(content) = message.content {
                match parse_message_html(theme, content) {
                    Ok(result) => {
                        message_row = message_row.push(result);
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
                        message_row = message_row.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        } else if message_type == "Text" {
            if let Some(content) = message.content {
                message_row = message_row.push(text(content));
            }
        } else {
            if let Some(content) = message.content {
                message_row = message_row.push(text(content));
            }
        }
    }

    contents_column = contents_column.push(message_row);

    // Files

    if !deleted {
        if let Some(properties) = &message.properties {
            if let Some(files) = &properties.files {
                let mut files_row = row![].spacing(10);

                for file in files {
                    let file_container = mouse_area(
                        container(
                            row![
                                svg("images/paperclip.svg").width(16).height(16),
                                text(file.file_name.clone().unwrap_or("File".to_string()))
                                    .color(theme.colors.text_link)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(8),
                        )
                        .style(|_| container::Style {
                            background: Some(theme.colors.primary3.into()),
                            border: border::rounded(6),
                            ..Default::default()
                        })
                        .padding(12),
                    )
                    .on_release(Message::DownloadFile(file.clone()));
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
                    if let Some(reaction_unicode) = reaction_val {
                        reaction_text = text(reaction_unicode.clone());
                    }

                    let reaction_container =
                        container(row![reaction_text, text(reacters)].spacing(4))
                            .style(|_| theme.stylesheet.primary_button)
                            .padding(Padding {
                                top: 3.0,
                                right: 3.0,
                                bottom: 3.0,
                                left: 5.0,
                            })
                            .align_y(Alignment::Center);
                    reactions_row = reactions_row.push(reaction_container);

                    are_reactions = true;
                }
            }
        }
    }

    if are_reactions {
        reactions_row = reactions_row.push(
            container(text("+"))
                .style(|_| theme.stylesheet.primary_button)
                .padding(Padding {
                    top: 3.0,
                    right: 5.0,
                    bottom: 3.0,
                    left: 5.0,
                }),
        );

        contents_column = contents_column.push(reactions_row);
    }

    // Actions container

    let mut action_container = container(row![]);

    // Fill the container if the message is being hovered.

    if is_hovered {
        action_container = container(
            container(
                container(
                    row![
                        svg("images/pencil.svg").width(17).height(17),
                        svg("images/reply.svg").width(21).height(21),
                        container(text("+").size(20))
                    ]
                    .align_y(Alignment::Center)
                    .spacing(8),
                )
                .padding(Padding {
                    top: 3.0,
                    right: 6.0,
                    bottom: 3.0,
                    left: 6.0,
                })
                .style(|_| theme.stylesheet.primary_button),
            )
            .padding(padding::right(10))
            .align_y(Alignment::Center),
        )
        .align_right(iced::Length::Fill);
    }

    let message_stack = stack!(
        mouse_area(
            container(contents_column)
                .style(move |_| container::Style {
                    background: if is_hovered {
                        Some(theme.colors.primary2_highlight.into())
                    } else {
                        Some(theme.colors.primary2.into())
                    },
                    border: border::rounded(4),
                    ..Default::default()
                })
                .padding(Padding {
                    top: 2.0,
                    right: 6.0,
                    bottom: 2.0,
                    left: 6.0,
                })
        )
        .on_enter(Message::ShowChatMessageOptions(message.id.clone().unwrap()))
        .on_exit(Message::StopShowChatMessageOptions(message.id.unwrap())),
        action_container
    );

    return Some(message_stack.into());
}
