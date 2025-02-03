use crate::Message;
use iced::widget::text;
use iced::Element;

pub fn login<'a>() -> Element<'a, Message> {
    text("Sign in to your account on the browser window").into()
}
