use iced::widget::{column, container, mouse_area, text};
use iced::Element;
use indexmap::IndexMap;

use crate::types::Emoji;
use crate::websockets::Presence;
use crate::Message;
use crate::{api, style};
use std::collections::HashMap;

use crate::api::{Conversation, Profile};
use crate::components::message::c_message;

pub fn c_conversation<'a>(
    theme: &'a style::Theme,
    messages: Vec<api::Message>,
    source_thread_id: String,
    conversation_id: String,
    show_replies: bool,
    emoji_map: &IndexMap<String, Emoji>,
    users: &HashMap<String, Profile>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
) -> Option<Element<'a, Message>> {
    let mut message_chain = column![].spacing(20);

    let first_message = messages.get(0).unwrap().clone();
    if let Some(message_element) = c_message(
        theme,
        &source_thread_id,
        first_message,
        emoji_map,
        users,
        me,
        user_presences,
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
            .on_release(Message::ToggleReplyOptions(conversation_id)),
        );
    }

    if show_replies && messages.len() > 1 {
        for message in messages.iter().skip(1).cloned() {
            if let Some(message_element) = c_message(
                theme,
                &source_thread_id,
                message,
                emoji_map,
                users,
                me,
                user_presences,
            ) {
                message_chain = message_chain.push(message_element);
            }
        }
    }
    Some(
        container(message_chain)
            .style(|_| theme.stylesheet.conversation)
            .width(iced::Length::Fill)
            .padding(20)
            .into(),
    )
}
