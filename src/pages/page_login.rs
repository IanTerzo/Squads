use crate::Message;
use iced::widget::{column, rich_text, span, text};
use iced::Element;

pub fn login<'a>(code: &'a Option<String>) -> Element<'a, Message> {
    let code = code.as_deref().unwrap_or("...");
    column![
        rich_text![
            span("Head over to "),
            span("https://aka.ms/devicelogin")
                .underline(true)
                .link(Message::LinkClicked("aka.ms/devicelogin".to_string())),
            span(" and enter the following code to authorize:")
        ],
        text(code)
    ]
    .into()
}
