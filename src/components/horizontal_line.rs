use iced::{
    widget::{container, space},
    Element, Length,
};

use crate::{style::Theme, Message};

pub fn c_horizontal_line(theme: &Theme, lenght: Length) -> Element<'_, Message> {
    container(space().width(lenght).height(1))
        .style(|_| container::Style {
            background: Some(theme.colors.primary3.into()),
            ..Default::default()
        })
        .into()
}
