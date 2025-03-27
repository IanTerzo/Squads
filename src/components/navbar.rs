use iced::widget::{container, row, svg, MouseArea};
use iced::{Element, Fill, Padding};

use crate::style;
use crate::{Message, Page, View};

pub fn c_navbar(theme: &style::Theme) -> Element<Message> {
    container(row![
        row![
            MouseArea::new(svg("images/chevron-left.svg").width(28).height(28))
                .on_press(Message::HistoryBack),
            MouseArea::new(svg("images/chevron-right.svg").width(28).height(28))
                .on_press(Message::HistoryForward),
        ],
        container(
            row![
                MouseArea::new(svg("images/house.svg").width(25).height(25)).on_press(
                    Message::Jump(Page {
                        view: View::Homepage,
                        current_team_id: None,
                        current_channel_id: None,
                        current_chat_id: None
                    })
                ),
                MouseArea::new(svg("images/message-square.svg").width(25).height(25)).on_press(
                    Message::Jump(Page {
                        view: View::Chat,
                        current_team_id: None,
                        current_channel_id: None,
                        current_chat_id: None
                    })
                )
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
    })
    .into()
}
