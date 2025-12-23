pub mod page_chat;
pub mod page_login;
pub mod page_team;
use crate::Message;
use iced::Element;
use iced::widget::{container, row};

pub fn app<'a>(
    sidebar: Element<'a, Message>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    row![sidebar, container(content)].into()
}
