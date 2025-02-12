use iced::widget::{container, scrollable, Column};
use iced::{border, Color, Element};

use crate::Message;

pub fn c_styled_scrollbar<'a>(content: Column<'a, Message>) -> Element<'a, Message> {
    scrollable(content)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(10)
                .spacing(10)
                .scroller_width(10),
        ))
        .style(|_, _| scrollable::Style {
            container: container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0).into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#444").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#666").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            gap: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
        })
        .into()
}
