use iced::widget::{column, container, mouse_area, text};
use iced::{border, Color, Element};

use crate::style::Stylesheet;
use crate::Message;
use std::collections::HashMap;

use crate::api::Conversation;
use crate::components::message::c_message;

pub fn c_conversation<'a>(
    theme: &'a Stylesheet,
    conversation: Conversation,
    show_replies: bool,
    emoji_map: &HashMap<String, String>,
) -> Option<Element<'a, Message>> {
    let mut message_chain = column![].spacing(20);

    let ordered_conversation: Vec<_> = conversation.messages.iter().rev().cloned().collect();

    let first_message = ordered_conversation.get(0).unwrap().clone();
    if let Some(message_element) = c_message(theme, first_message, emoji_map) {
        message_chain = message_chain.push(message_element);
    } else {
        return None;
    }

    if ordered_conversation.len() > 1 {
        message_chain = message_chain.push(
            mouse_area(
                text(if show_replies {
                    "Hide replies"
                } else {
                    "Show replies"
                })
                .color(Color::from_rgb(0.4, 0.5961, 0.851))
                .size(14),
            )
            .on_release(Message::ToggleReplyOptions(conversation.id)),
        );
    }

    if show_replies && ordered_conversation.len() > 1 {
        for message in ordered_conversation.iter().skip(1).cloned() {
            if let Some(message_element) = c_message(theme, message, emoji_map) {
                message_chain = message_chain.push(message_element);
            }
        }
    }
    Some(
        container(message_chain)
            .style(|_| theme.conversation)
            .width(iced::Length::Fill)
            .padding(20)
            .into(),
    )
}
