use crate::Message;
use iced::widget::{column, container, rich_text, span, text_input};
use iced::{Alignment, Element, Length};

pub fn login<'a>(theme: &'a crate::style::Theme, code: &'a Option<String>) -> Element<'a, Message> {
    let code = code.as_deref().unwrap_or("...");
    column![
        container(rich_text![
            span("Head over to "),
            span("aka.ms/devicelogin")
                .underline(true)
                .link(Message::LinkClicked(
                    "https://aka.ms/devicelogin".to_string()
                )),
            span(" and enter the following code to authorize:")
        ])
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .padding(30),
        container(
            text_input("", code)
                .style(|_, _| text_input::Style {
                    background: theme.colors.background.into(),
                    border: iced::Border {
                        color: theme.colors.not_set,
                        width: 0.0,
                        radius: 0.0.into()
                    },
                    placeholder: theme.colors.not_set,
                    icon: theme.colors.not_set,
                    value: theme.colors.text,
                    selection: theme.colors.text_selection
                })
                .align_x(Alignment::Center)
                .padding(0)
        )
        .width(Length::Fill)
    ]
    .into()
}
