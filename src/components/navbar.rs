use std::f32::consts::PI;

use iced::widget::{container, image, row, svg, MouseArea};
use iced::{Color, Element, Fill, Padding, Rotation};

use crate::{Message, Page, View};

pub fn c_navbar() -> Element<'static, Message> {
    container(row![
        row![
            MouseArea::new(image("images/icons8-back-64.png").width(31).height(31))
                .on_press(Message::HistoryBack),
            image("images/icons8-back-64.png")
                .width(31)
                .height(31)
                .rotation(Rotation::Floating(PI.into())) //.padding(padding::bottom(20))
        ],
        container(
            row![
                MouseArea::new(svg("images/icons8-home.svg").width(31).height(31)).on_press(
                    Message::Jump(Page {
                        view: View::Homepage,
                        current_team_id: "0".to_string(),
                        current_channel_id: "0".to_string(),
                        show_conversations: false,
                    })
                ),
                MouseArea::new(image("images/icons8-chat-96.png").width(31).height(31)).on_press(
                    Message::Jump(Page {
                        view: View::Chat,
                        current_team_id: "0".to_string(),
                        current_channel_id: "0".to_string(),
                        show_conversations: false,
                    })
                )
            ]
            .spacing(5)
        )
        .align_right(Fill)
    ])
    .style(|_| container::Style {
        background: Some(
            Color::parse("#333")
                .expect("Background color is invalid.")
                .into(),
        ),
        //border: border::rounded(10),
        ..Default::default()
    })
    .width(Fill)
    .center_y(45)
    .padding(Padding::from([0, 20]))
    .into()
}
