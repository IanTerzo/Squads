use crate::api::Chat;
use crate::Message;
use iced::widget::text;
use iced::Element;

pub fn chat<'a>(chats: Vec<Chat>) -> Element<'a, Message> {
    text("Chat").into()
}
