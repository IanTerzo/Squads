pub mod page_chat;
pub mod page_login;
pub mod page_team;
use crate::Message;
use iced::widget::stack;
use iced::widget::{container, row};
use iced::Element;

pub fn app<'a>(
    sidebar: Element<'a, Message>,
    content: Element<'a, Message>,
    overlay: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    if let Some(overlay) = overlay {
        stack![row![sidebar, container(content)], overlay].into()
    } else {
        stack![row![sidebar, container(content)]].into()
    }
}
