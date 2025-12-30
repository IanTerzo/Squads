pub mod page_activity;
pub mod page_chat;
pub mod page_login;
pub mod page_team;
use crate::Message;
use iced::Element;
use iced::widget::{row, stack};

pub fn app<'a>(
    sidebar: Element<'a, Message>,
    content: Element<'a, Message>,
    overlay: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    if let Some(overlay) = overlay {
        stack![row![sidebar, content], overlay].into()
    } else {
        stack![row![sidebar, content]].into()
    }
}
