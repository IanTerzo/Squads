use iced::widget::{column, container, row, svg, MouseArea, Space};
use iced::{Element, Fill, Length, Padding};

use crate::style;
use crate::{Message, Page, View};

pub fn c_navbar(theme: &style::Theme) -> Element<Message> {
    column![
        container(row![
            row![
                MouseArea::new(svg("images/chevron-left.svg").width(28).height(28))
                    .on_release(Message::HistoryBack),
                MouseArea::new(svg("images/chevron-right.svg").width(28).height(28))
                    .on_release(Message::HistoryForward),
            ],
            container(
                row![
                    MouseArea::new(svg("images/house.svg").width(25).height(25))
                        .on_release(Message::OpenHome),
                    MouseArea::new(svg("images/message-square.svg").width(25).height(25))
                        .on_enter(Message::PrefetchCurrentChat)
                        .on_release(Message::OpenCurrentChat)
                ]
                .spacing(10)
            )
            .align_right(Fill)
        ])
        .style(|_| theme.stylesheet.navbar)
        .width(Fill)
        .center_y(40)
        .padding(Padding {
            top: 4.0,
            right: 20.0,
            bottom: 0.0,
            left: 20.0,
        }),
        container(Space::new(Length::Fill, 1)).style(|_| container::Style {
            background: Some(theme.colors.primary3.into()),
            ..Default::default()
        }),
    ]
    .into()
}
