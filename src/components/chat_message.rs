use crate::api::Profile;
use crate::components::cached_image::c_cached_image;
use crate::parsing::{parse_card_html, parse_message_html};
use crate::style;
use crate::Message;
use iced::widget::{column, container, row, text};
use iced::{font, padding, Alignment, Element, Font, Padding};
use std::collections::HashMap;
use unicode_properties::UnicodeEmoji;

const LOG_THREAD_ACTIVITY: bool = false;

pub fn c_chat_message<'a>(
    theme: &'a style::Theme,
    message: crate::api::Message,
    emoji_map: &HashMap<String, String>,
    users: &HashMap<String, Profile>,
) -> Option<Element<'a, Message>> {
    if let Some(message_type) = message.message_type.clone() {
        if message_type.contains("ThreadActivity") && !LOG_THREAD_ACTIVITY {
            return None;
        }
    }

    let mut message_row = row![].spacing(12);

    let mut message_column = column![].spacing(3);
    let mut contents_column = column![].spacing(2);

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    // Message info bar

    if let Some(message_type) = message.message_type.clone() {
        if message_type == "RichText/Html" || message_type == "Text" {
            // The message.im_display_name value is useless. Some messages don't have it and it can be set completely arbitrarily by the client. Instead, Teams matches the displayname from the user id.
            if let Some(user_id) = message.from {
                let profile = users.get(&user_id.replace("8:orgid:", ""));
                if let Some(profile) = profile {
                    let display_name = profile.display_name.clone().unwrap();

                    let identifier = user_id.clone().replace(":", "");

                    let user_picture = c_cached_image(
                        identifier.clone(),
                        Message::FetchUserImage(identifier, user_id, display_name.clone()),
                        31.0,
                        31.0,
                    );

                    message_row =
                        message_row.push(container(user_picture).padding(padding::top(3)));
                    message_info = message_info.push(text!("{}", display_name).font(Font {
                        weight: font::Weight::Bold,
                        ..Default::default()
                    }));
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

    contents_column = contents_column.push(message_info);

    // Message content

    let deleted = if let Some(properties) = message.properties.clone() {
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

    message_column = message_column.push(contents_column);

    // Message reactions

    if !deleted {
        let mut reactions_row = row![].spacing(10);

        if let Some(properties) = message.properties {
            if let Some(reactions) = properties.emotions {
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

    message_row = message_row.push(message_column);

    return Some(
        container(message_row)
            .width(iced::Length::Fill)
            .padding(8)
            .into(),
    );
}
