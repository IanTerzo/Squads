use crate::Message;
use iced::widget::{column, container, rich_text, span, text_input};
use iced::{Alignment, Element, Font, Length};

pub fn login<'a>(theme: &'a crate::style::Theme, code: &'a Option<String>) -> Element<'a, Message> {
    let code = code.as_deref().unwrap_or("...");
    column![
        container(
            rich_text![
                span::<String, Font>("Head over to "),
                span("aka.ms/devicelogin")
                    .color(theme.colors.text_link)
                    .link("https://aka.ms/devicelogin".to_string()),
                span(" and enter the following code to authorize:")
            ]
            .on_link_click(|link| Message::LinkClicked(link))
        )
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
