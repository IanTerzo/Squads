use crate::{Message, style::Theme};
use iced::{
    Border, Element, Padding, border,
    widget::{container, text},
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
            top: 6.0,
            bottom: 6.0,
            right: 8.0,
            left: 8.0,
        })
        .into()
}
