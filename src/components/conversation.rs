use iced::widget::{column, container, mouse_area, text};
use iced::{Element, border};
use indexmap::IndexMap;

use crate::Message;
use crate::types::Emoji;
use crate::websockets::Presence;
use crate::{api, style};
use std::collections::HashMap;

use crate::api::Profile;
use crate::components::message::c_message;

pub fn c_conversation<'a>(
    theme: &'a style::Theme,
    messages: Vec<api::Message>,
    source_thread_id: String,
    conversation_id: String,
    show_replies: bool,
    emoji_map: &'a IndexMap<String, Emoji>,
    search_emojis_input_value: &String,
    users: &HashMap<String, Profile>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
    show_plus_emoji_picker: &bool,
    emoji_picker_message_id: &Option<String>,
    window_size: &(f32, f32),
) -> Option<Element<'a, Message>> {
    let mut message_chain = column![].spacing(20);

    let first_message = messages.get(0).unwrap().clone();
    if let Some(message_element) = c_message(
        theme,
        source_thread_id.clone(),
        first_message,
        emoji_map,
        search_emojis_input_value,
        users,
        me,
        user_presences,
        show_plus_emoji_picker,
        emoji_picker_message_id,
        window_size,
    ) {
        message_chain = message_chain.push(message_element);
    } else {
        return None;
    }

    if messages.len() > 1 {
        message_chain = message_chain.push(
            mouse_area(
                text(if show_replies {
                    "Hide replies"
                } else {
                    "Show replies"
                })
                .color(theme.colors.text_link)
                .size(14),
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::ToggleReplyOptions(conversation_id)),
        );
    }

    if show_replies && messages.len() > 1 {
        for message in messages.iter().skip(1).cloned() {
            if let Some(message_element) = c_message(
                theme,
                source_thread_id.clone(),
                message,
                emoji_map,
                search_emojis_input_value,
                users,
                me,
                user_presences,
                show_plus_emoji_picker,
                emoji_picker_message_id,
                window_size,
            ) {
                message_chain = message_chain.push(message_element);
            }
        }
    }
    Some(
        container(message_chain)
            .style(|_| container::Style {
                background: Some(theme.colors.foreground.into()),
                border: border::rounded(8),
                ..Default::default()
            })
            .width(iced::Length::Fill)
            .padding(20)
            .into(),
    )
}
