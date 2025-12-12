use crate::{style::Theme, Message};
use iced::{
    border,
    widget::{container, text},
    Border, Element, Padding,
};

pub fn c_tooltip<'a>(theme: &'a Theme, message: &'a str) -> Element<'a, Message> {
    container(text(message).wrapping(text::Wrapping::WordOrGlyph))
        .max_width(150)
        .style(|_| container::Style {
            background: Some(theme.colors.tooltip.into()),
            border: Border {
                color: theme.colors.line,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(Padding {
            top: 8.0,
            bottom: 10.0,
            right: 10.0,
            left: 8.0,
        })
        .into()
}
