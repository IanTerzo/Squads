use crate::api::Chat;
use crate::Message;
use iced::widget::text;
use iced::Element;
use iced_widget::{column, row, scrollable};

pub fn chat<'a>(chats: Vec<Chat>) -> Element<'a, Message> {
    let mut chats_column = column![];
    for chat in chats {
        let mut title = "Chat".to_string();
        if let Some(title_spec) = chat.title {
            title = title_spec;
        }
        chats_column = chats_column.push(text(title));
    }
    scrollable(chats_column).into()
}
