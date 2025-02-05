use crate::Message;
use iced::widget::text;
use iced::Element;

pub fn chat<'a>() -> Element<'a, Message> {
    text("Chat").into()
}
