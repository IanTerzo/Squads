pub mod page_chat;
pub mod page_home;
pub mod page_login;
pub mod page_team;

use crate::components::navbar::c_navbar;

use crate::style;
use crate::Message;
use iced::widget::stack;
use iced::widget::{column, container};
use iced::Element;

pub fn app<'a>(
    theme: &'a style::Theme,
    content: Element<'a, Message>,
    overlay: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    if let Some(overlay) = overlay {
        stack![column![c_navbar(theme), container(content)], overlay,].into()
    } else {
        stack![column![c_navbar(theme), container(content)]].into()
    }
}
