pub mod page_chat;
pub mod page_login;
pub mod page_team;
use crate::Message;
use crate::widgets::centered_overlay::centered_overlay;
use iced::Element;
use iced::widget::{container, row, stack};

pub fn app<'a>(
    sidebar: Element<'a, Message>,
    content: Element<'a, Message>,
    overlay: Option<Element<'a, Message>>,
    window_size: (f32, f32),
) -> Element<'a, Message> {
    if let Some(overlay) = overlay {
        stack![
            row![sidebar, container(content)],
            centered_overlay(overlay, window_size, 0.9),
        ]
        .into()
    } else {
        stack![row![sidebar, container(content)]].into()
    }
}
