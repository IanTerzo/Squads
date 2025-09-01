use crate::api::Profile;
use crate::components::cached_image::c_cached_image;
use crate::components::picture_and_status::c_picture_and_status;
use crate::parsing::{parse_card_html, parse_message_html};
use crate::websockets::Presence;
use crate::Message;
use crate::{style, utils};
use iced::overlay::menu::{Menu, State};
use iced::widget::{column, container, mouse_area, row, svg, text};
use iced::{border, font, Alignment, Element, Font, Padding};
use std::collections::HashMap;

const LOG_THREAD_ACTIVITY: bool = false;

pub fn c_message<'a>(
    theme: &'a style::Theme,
    message: crate::api::Message,
    emoji_map: &HashMap<String, String>,
    users: &HashMap<String, Profile>,
    user_presences: &'a HashMap<String, Presence>,
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
            // The message.im_display_name value is useless. Some messages don't have it and it can be set completely arbitrarily by the client. Instead, Teams matches the displayname from the user id.
            if let Some(user_id) = message.from {
                let profile = users.get(&user_id.replace("8:orgid:", ""));
                if let Some(profile) = profile {
                    let display_name = profile
                        .display_name
                        .clone()
                        .unwrap_or("Unknown User".to_string());

                    let identifier = user_id.clone().replace(":", "");

                    let user_picture = c_cached_image(
                        identifier.clone(),
                        Message::FetchUserImage(identifier, user_id.clone(), display_name.clone()),
                        31.0,
                        31.0,
                    );

                    let presence = user_presences.get(&user_id);

                    let picture_and_status =
                        c_picture_and_status(theme, user_picture, presence, (31.0, 31.0));
                    message_info = message_info.push(picture_and_status);
                    message_info = message_info.push(text!("{}", display_name));
                } else {
                    message_info = message_info.push(text("Unknown User"));
                }
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

        message_info = message_info.push(text(date).size(14).color(theme.colors.demo_text));
        message_info = message_info.push(text(time).size(14).color(theme.colors.demo_text));
    }

    message_column = message_column.push(message_info);

    // Message subject

    if let Some(properties) = &message.properties {
        if let Some(title) = &properties.title {
            if title != "" {
                let trimmed_title = title.trim_start();
                let mut text_row = row![];
                if trimmed_title != "" {
                    text_row =
                        text_row.push(text(trimmed_title.to_string()).size(18).font(font::Font {
                            weight: font::Weight::Bold,
                            ..Default::default()
                        }));
                    if let Some(subject) = &properties.subject {
                        let trimmed_subject = subject.trim_start();
                        if trimmed_subject != "" {
                            message_column = message_column.push(
                                text(trimmed_subject.to_string()).font(font::Font {
                                    weight: font::Weight::Bold,
                                    ..Default::default()
                                }),
                            );
                        }
                    }
                }
                message_column = message_column.push(text_row);
            } else {
                if let Some(subject) = &properties.subject {
                    let trimmed_subject = subject.trim_start();
                    if trimmed_subject != "" {
                        message_column = message_column.push(
                            text(trimmed_subject.to_string()).font(font::Font {
                                weight: font::Weight::Bold,
                                ..Default::default()
                            }),
                        );
                    }
                }
            }
        } else if let Some(subject) = &properties.subject {
            let trimmed_subject = subject.trim_start();
            // Edgecase
            if trimmed_subject != "" {
                message_column =
                    message_column.push(text(trimmed_subject.to_string()).font(font::Font {
                        weight: font::Weight::Bold,
                        ..Default::default()
                    }));
            }
        }
    }

    // Cards

    if let Some(properties) = &message.properties {
        if let Some(cards) = &properties.cards {
            for card in cards {}
        }
    }

    // Message content

    let deleted = if let Some(properties) = &message.properties {
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
                match parse_message_html(theme, content) {
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
                message_column = message_column.push(text(content));
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
                }
            }
        }

        let add_reaction_container =
            container(row![text("+")].spacing(4).padding(Padding::from([0, 3])))
                .style(|_| theme.stylesheet.primary_button)
                .padding(3)
                .align_y(Alignment::Center);

        reactions_row = reactions_row.push(add_reaction_container);

        message_column = message_column.push(reactions_row);
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
                            background: Some(theme.colors.primary2.into()),
                            border: border::rounded(6),
                            ..Default::default()
                        })
                        .padding(12),
                    )
                    .on_release(Message::DownloadFile(file.clone()));
                    files_row = files_row.push(file_container);
                }

                message_column = message_column.push(files_row.wrap());
            }
        }
    }

    return Some(message_column.into());
}
