use std::f32::consts::PI;

use iced::widget::{container, image, row, svg, MouseArea};
use iced::{Color, Element, Fill, Padding, Rotation};

use crate::style::Stylesheet;
use crate::{Message, Page, View};

pub fn c_navbar(theme: &Stylesheet) -> Element<Message> {
    container(row![
        row![
            MouseArea::new(svg("images/chevron-left.svg").width(28).height(28))
                .on_press(Message::HistoryBack),
            svg("images/chevron-right.svg").width(28).height(28)
        ],
        container(
            row![
                MouseArea::new(svg("images/house.svg").width(25).height(25)).on_press(
                    Message::Jump(Page {
                        view: View::Homepage,
                        current_team_id: "0".to_string(),
                        current_channel_id: "0".to_string(),
                        show_conversations: false,
                    })
                ),
                MouseArea::new(svg("images/message-square.svg").width(25).height(25)).on_press(
                    Message::Jump(Page {
                        view: View::Chat,
                        current_team_id: "0".to_string(),
                        current_channel_id: "0".to_string(),
                        show_conversations: false,
                    })
                )
            ]
            .spacing(10)
        )
        .align_right(Fill)
    ])
    .style(|_| theme.navbar)
    .width(Fill)
    .center_y(45)
    .padding(Padding::from([0, 20]))
    .into()
}
