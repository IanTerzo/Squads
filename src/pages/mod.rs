pub mod page_chat;
pub mod page_home;
pub mod page_login;
pub mod page_team;

use crate::components::navbar::c_navbar;

use crate::style::Stylesheet;
use crate::Message;
use iced::widget::{column, container};
use iced::Element;

pub fn app<'a>(theme: &'a Stylesheet, content: Element<'a, Message>) -> Element<'a, Message> {
    column![c_navbar(theme), container(content).padding(20)].into()
}
