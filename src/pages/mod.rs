pub mod page_home;
pub mod page_login;
pub mod page_team;

use crate::components::navbar::navbar;

use crate::Message;
use iced::widget::{column, container};
use iced::Element;

pub fn app<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    column![navbar(), container(content).padding(20)].into()
}
