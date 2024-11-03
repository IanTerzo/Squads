use iced::widget::{container, row};
use iced::{border, Color, Element};

use crate::Message;

pub fn navbar() -> Element<'static, Message> {
    container(
        container(row![])
            .style(|_| container::Style {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                ..Default::default()
            })
            .width(4000) // Hotfix?
            .height(45), // Should be 50?
    )
    .padding(20)
    .width(4000) // Hotfix?
    .into()
}
