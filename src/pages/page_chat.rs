use std::collections::HashMap;

use crate::api::{fetch_short_profile, AccessToken, Chat, ShortProfile};
use crate::Message;
use iced::widget::text;
use iced::Element;
use iced_widget::{column, row, scrollable};

pub fn chat<'a>(
    chats: Vec<Chat>,
    org_users: HashMap<String, ShortProfile>,
) -> Element<'a, Message> {
    let mut chats_column = column![];
    for chat in chats {
        let mut title = "Chat".to_string();
        if let Some(title_spec) = chat.title {
            title = title_spec;
        } else {
            let mut member_names = vec![];
            for member in chat.members {
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

            title = member_names.join(", ")
        }
        chats_column = chats_column.push(text(title));
    }
    scrollable(chats_column).into()
}
