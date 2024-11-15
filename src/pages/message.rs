use std::f32::consts::PI;

use iced::widget::{container, image, row, svg};
use iced::{border, padding, Color, Element, Fill, Padding, Rotation};

use crate::Message;


pub fn message() -> Element<'static, Message> {
    container(row![])
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
