use std::collections::HashMap;

use crate::api::{self, Chat, Profile};
use crate::components::{
    cached_image::c_cached_image, chat_message::c_chat_message, message_area::c_message_area,
};
use crate::style;
use crate::utils::truncate_name;
use crate::Message;

use iced::widget::scrollable::Id;
use iced::widget::text_editor::Content;
use iced::widget::{column, container, mouse_area, row, Space};
use iced::widget::{scrollable, text};
use iced::{padding, Alignment, Element, Length};

pub fn chat<'a>(
    theme: &'a style::Theme,
    chats: &Vec<Chat>,
    conversation: &Option<&Vec<api::Message>>,
    emoji_map: &HashMap<String, String>,
    users: &HashMap<String, Profile>,
    user_id: String,
    message_area_content: &'a Content,
    message_area_height: &f32,
) -> Element<'a, Message> {
    let mut chats_column = column![].spacing(8.5);

    for chat in chats {
        let mut picture = Space::new(0, 0).into(); // Temporary

        let mut title = "Chat".to_string();

        if let Some(chat_title) = &chat.title {
            title = truncate_name(chat_title.clone(), 20);
        } else if chat.members.len() == 2 {
            for member in chat.members.clone() {
                let member_id = member.mri.replace("8:orgid:", "");
                if member_id != user_id {
                    if let Some(user_profile) = users.get(&member_id) {
                        if let Some(display_name) = user_profile.clone().display_name {
                            title = truncate_name(display_name.clone(), 24);
                        } else {
                            title = "Unknown User".to_string();
                        }
                    } else {
                        title = "Unknown User".to_string();
                    }
                }
            }
        } else {
            let mut member_names = vec![];
            for member in chat.members.clone() {
                let member_id = member.mri.replace("8:orgid:", "");
                if member_id != user_id {
                    if let Some(user_profile) = users.get(&member_id) {
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
            }

            title = truncate_name(member_names.join(", "), 24);
        }

        if let Some(chat_picture) = &chat.picture {
            let url = chat_picture.replace("URL@", "");
            let identifier = url.replace("https:", "").replace("/", "").replace(":", "");
            picture = c_cached_image(
                identifier.clone(),
                Message::AuthorizeImage(url, identifier),
                28.0,
                28.0,
            );
        } else {
            let mut member_profiles = vec![];
            for member in &chat.members {
                let member_id = member.mri.replace("8:orgid:", "");
                if member_id != user_id {
                    if let Some(user_profile) = users.get(&member_id) {
                        if let Some(display_name) = user_profile.clone().display_name {
                            member_profiles.push((member.mri.clone(), display_name.clone()));
                        }
                    }
                }
            }

            let identifier = member_profiles
                .iter()
                .map(|(a, _)| a)
                .cloned()
                .collect::<Vec<_>>()
                .join("-");

            picture = c_cached_image(
                identifier.clone(),
                Message::FetchMergedProfilePicture(identifier, member_profiles),
                31.0,
                31.0,
            );
        }

        let chat_item = mouse_area(
            container(
                row![picture, text(title)]
                    .spacing(10)
                    .padding(padding::left(10))
                    .align_y(Alignment::Center),
            )
            .style(|_| theme.stylesheet.list_tab)
            .center_y(47)
            .width(220),
        )
        .on_enter(Message::PrefetchChat(chat.id.clone()))
        .on_release(Message::OpenChat(chat.id.clone()));

        chats_column = chats_column.push(chat_item);
    }
    let chats_scrollable = scrollable(chats_column)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(8)
                .spacing(10)
                .scroller_width(8),
        ))
        .style(|_, _| theme.stylesheet.scrollable);

    let mut message_column = column![].spacing(10);

    if let Some(conversation) = conversation {
        let ordered_conversation: Vec<_> = conversation.iter().rev().cloned().collect();

        for message in ordered_conversation {
            if let Some(message_element) = c_chat_message(theme, message, emoji_map, users) {
                message_column = message_column.push(message_element);
            }
        }
    };

    let conversation_scrollbar = container(
        scrollable(message_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(8)
                    .spacing(10)
                    .scroller_width(8),
            ))
            .style(|_, _| theme.stylesheet.chat_scrollable)
            .id(Id::new("conversation_column")),
    )
    .height(Length::Fill);

    let message_area = c_message_area(
        theme,
        message_area_content,
        message_area_height,
        "chat".to_string(),
    );
    let content_page = column![conversation_scrollbar, message_area].spacing(7);

    row![chats_scrollable, content_page].spacing(10).into()
}
