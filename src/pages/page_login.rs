use crate::Message;
use iced::widget::{column, container, rich_text, span, text, text_input};
use iced::{Alignment, Color, Element, Font, Length};

pub fn login<'a>(theme: &'a crate::style::Theme, code: &'a Option<String>, session_expired: bool) -> Element<'a, Message> {
    let code = code.as_deref().unwrap_or("...");
    let mut content = column![];

    if session_expired {
        content = content.push(
            container(
                text("Your session has expired. Please sign in again.")
                    .color(Color::from_rgb(1.0, 0.6, 0.2))
            )
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .padding(20),
        );
    }

    content = content
        .push(
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
        )
        .push(
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
            .width(Length::Fill),
        );

    content.into()
}
