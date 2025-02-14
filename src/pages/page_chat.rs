use std::collections::HashMap;

use crate::api::{Chat, ShortProfile};
use crate::components::{cached_image::c_cached_image, styled_scrollbar::c_styled_scrollbar};
use crate::utils::truncate_name;
use crate::Message;

use iced::widget::text;
use iced::{border, padding, Alignment, Color, Element};
use iced_widget::{column, container, row, Space};

pub fn chat<'a>(
    chats: Vec<Chat>,
    org_users: HashMap<String, ShortProfile>,
    user_id: String,
) -> Element<'a, Message> {
    let mut chats_column = column![].spacing(8.5);

    for chat in chats {
        let mut picture = Space::new(0, 0).into(); // Temporary

        let mut title = "Chat".to_string();

        if let Some(chat_picture) = chat.picture {
            let url = chat_picture.replace("URL@", "");
            let identifier = url
                .replace("https://eu-prod.asyncgw.teams.microsoft.com/v1/objects", "")
                .replace("/", "");
            picture = c_cached_image(identifier, Message::FetchAvatar(url), 28.0, 28.0);
        }

        if let Some(chat_title) = chat.title {
            title = truncate_name(chat_title, 20);
        } else if chat.members.len() == 2 {
            for member in chat.members {
                if member.mri != format!("8:{user_id}") {
                    if let Some(user_profile) = org_users.get(&member.mri) {
                        if let Some(display_name) = user_profile.clone().display_name {
                            title = truncate_name(display_name.clone(), 24);
                            picture = c_cached_image(
                                member.mri.clone(),
                                Message::FetchUserImage(member.mri, display_name),
                                31.0,
                                31.0,
                            );
                        } else {
                            title = "Unknown User".to_string();
                        }
                    } else {
                        // This should never happen
                        title = "Unknown User".to_string();
                    }
                }
            }
        } else {
            let mut member_names = vec![];
            for member in chat.members.clone() {
                if let Some(user_profile) = org_users.get(&member.mri) {
                    if let Some(display_name) = user_profile.clone().display_name {
                        member_names.push(display_name);
                    } else {
                        member_names.push("Unknown User".to_string());
                    }
                } else {
                    // This should never happen
                    member_names.push("Unknown User".to_string());
                }
            }

            title = truncate_name(member_names.join(", "), 24);
        }

        let chat_item = container(
            row![picture, text(title)]
                .spacing(10)
                .padding(padding::left(10))
                .align_y(Alignment::Center),
        )
        .style(|_| container::Style {
            background: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
            border: border::rounded(8),
            ..Default::default()
        })
        .center_y(47)
        .width(220);

        chats_column = chats_column.push(chat_item);
    }
    let chats_scrollable = c_styled_scrollbar(chats_column);

    row![chats_scrollable, "Hello, chat"].into()
}
