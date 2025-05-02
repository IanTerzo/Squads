use crate::api::Profile;
use crate::components::cached_image::c_cached_image;
use crate::parsing::{parse_card_html, parse_message_html};
use crate::style;
use crate::websockets::Presence;
use crate::widgets::circle::circle;
use crate::Message;
use base64::display;
use iced::widget::{column, container, mouse_area, row, stack, svg, text};
use iced::{border, font, padding, Alignment, Element, Font, Length, Padding};
use std::collections::HashMap;
use unicode_properties::UnicodeEmoji;

const LOG_THREAD_ACTIVITY: bool = false;

pub fn c_chat_message<'a>(
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

    let mut message_row = row![].spacing(3);

    let mut contents_column = column![].spacing(4);

    // Message info bar

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" || message_type == "Text" {
            if let Some(user_id) = message.from {
                let display_name =
                    if let Some(profile) = users.get(&user_id.replace("8:orgid:", "")) {
                        profile.display_name.clone().unwrap()
                    } else {
                        message.im_display_name.unwrap()
                    };

                let presence = user_presences.get(&user_id);

                let identifier = user_id.clone().replace(":", "");

                let user_picture = c_cached_image(
                    identifier.clone(),
                    Message::FetchUserImage(identifier, user_id, display_name.clone()),
                    31.0,
                    31.0,
                );

                message_row = message_row.push(container(stack![
                    container(user_picture).padding(Padding {
                        top: 7.0,
                        right: 11.0,
                        bottom: 4.0,
                        left: 8.0,
                    }),
                    container(circle(
                        5.5,
                        if let Some(presence) = presence {
                            if let Some(activity) = &presence.presence.activity {
                                match activity.as_str() {
                                    "Available" => theme.colors.status_available,
                                    "Busy" => theme.colors.status_busy,
                                    "DoNotDisturb" => theme.colors.status_busy,
                                    "InACall" => theme.colors.status_busy,
                                    "Presenting" => theme.colors.status_busy,
                                    "Away" => theme.colors.status_away,
                                    "BeRightBack" => theme.colors.status_away,
                                    _ => theme.colors.status_offline,
                                }
                            } else {
                                theme.colors.status_offline
                            }
                        } else {
                            theme.colors.status_offline
                        }
                    ))
                    .padding(Padding {
                        top: 30.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 32.0
                    })
                ]));
                message_info = message_info.push(text!("{}", display_name).font(Font {
                    weight: font::Weight::Bold,
                    ..Default::default()
                }));
            } else {
                if let Some(display_name) = message.im_display_name {
                    message_info = message_info.push(text(display_name));
                } else {
                    message_info = message_info.push(text("Unknown User"));
                }
            }
        } else if message_type == "RichText/Media_Card" {
            if let Some(display_name) = message.im_display_name {
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

    contents_column = contents_column.push(message_info);

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
            if let Some(content) = message.content {
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
            if let Some(content) = message.content {
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
            if let Some(content) = message.content {
                let mut text_row = row![];

                for c in content.chars() {
                    if c.is_emoji_char() {
                        text_row = text_row.push(text(c).font(Font::with_name("Twemoji")));
                    } else {
                        text_row = text_row.push(text(c));
                    }
                }

                contents_column = contents_column.push(text_row);
            }
        } else {
            if let Some(content) = message.content {
                contents_column = contents_column.push(text(content));
            }
        }
    }

    // Message reactions

    let mut reactions_row = row![]
        .spacing(8)
        .padding(padding::left(60))
        .align_y(Alignment::End)
        .height(Length::Fill);

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
                    let font = Font::with_name("Twemoji");

                    let reaction_val = emoji_map.get(&reaction.key);
                    if let Some(reaction_unicode) = reaction_val {
                        reaction_text = text(reaction_unicode.clone()).font(font);
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
                            background: Some(theme.colors.primary2.into()),
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

    message_row = message_row.push(container(contents_column).width(Length::Fill));

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
    }
    // Actions container

    let mut action_container = container(row![]);

    // Fill the container if the message is being hovered.

    let is_hovered = chat_message_options
        .get(&message.id.clone().unwrap())
        .unwrap_or(&false)
        .to_owned();

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
        container(
            mouse_area(
                container(message_row)
                    .style(|_| container::Style {
                        background: Some(theme.colors.primary1.into()),
                        border: border::rounded(4),
                        ..Default::default()
                    })
                    .padding(Padding {
                        top: 13.0,
                        right: 6.0,
                        bottom: 13.0,
                        left: 6.0,
                    })
            )
            .on_enter(Message::ShowChatMessageOptions(message.id.clone().unwrap()))
            .on_exit(Message::StopShowChatMessageOptions(message.id.unwrap()))
        )
        .padding(Padding {
            top: 5.0,
            right: 0.0,
            bottom: if are_reactions { 17.0 } else { 0.0 },
            left: 0.0
        }),
        container(reactions_row),
        action_container
    );

    return Some(message_stack.into());
}
