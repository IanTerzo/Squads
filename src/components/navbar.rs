use iced::widget::{column, container, row, svg, MouseArea, Space};
use iced::{Element, Fill, Length, Padding};

use crate::{style, utils};
use crate::{Message, Page, View};

pub fn c_navbar(theme: &style::Theme) -> Element<Message> {
    column![
        container(row![
            row![
                MouseArea::new(svg(utils::get_image_dir().join("chevron-left.svg")).width(23).height(23))
                    .on_release(Message::HistoryBack),
                MouseArea::new(svg(utils::get_image_dir().join("chevron-right.svg")).width(23).height(23))
                    .on_release(Message::HistoryForward),
            ],
            container(
                row![
                    MouseArea::new(svg(utils::get_image_dir().join("house.svg")).width(20).height(20))
                        .on_release(Message::OpenHome),
                    MouseArea::new(svg(utils::get_image_dir().join("message-square.svg")).width(20).height(20))
                        .on_enter(Message::PrefetchCurrentChat)
                        .on_release(Message::OpenCurrentChat)
                ]
                .spacing(10)
            )
            .align_right(Fill)
        ])
        .style(|_| theme.stylesheet.navbar)
        .width(Fill)
        .center_y(35)
        .padding(Padding {
            top: 2.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        }),
        container(Space::new(Length::Fill, 1)).style(|_| container::Style {
            background: Some(theme.colors.primary3.into()),
            ..Default::default()
        }),
    ]
    .into()
}
