use iced::widget::text_editor::Content;
use iced::widget::{column, container, mouse_area, rich_text, row, span, svg, text, text_editor};
use iced::{font, padding, Alignment, Element, Font, Length, Padding};

use crate::style;
use crate::Message;

pub fn c_message_area<'a>(
    theme: &'a style::Theme,
    message_area_content: &'a Content,
    message_area_height: &f32,
) -> Element<'a, Message> {
    container(
        container(column![
            container(
                row![
                    row![
                        container(text("Write"))
                            .style(|_| theme.stylesheet.message_area_tab)
                            .padding(3)
                            .align_y(Alignment::Center),
                        container(text("Preview"))
                            .style(|_| theme.stylesheet.message_area_tab)
                            .padding(3)
                            .align_y(Alignment::Center)
                    ]
                    .spacing(8),
                    container(
                        row![
                            row![
                                rich_text![span("B").font(Font {
                                    weight: font::Weight::Bold,
                                    ..Default::default()
                                })]
                                .size(20),
                                rich_text![span("I").font(Font {
                                    style: font::Style::Italic,
                                    ..Default::default()
                                })]
                                .size(20),
                                rich_text![span("U").underline(true)].size(20),
                                rich_text![span("S").strikethrough(true)].size(20)
                            ]
                            .spacing(8),
                            row![
                                svg("images/list.svg").width(23).height(23),
                                svg("images/list-ordered.svg").width(23).height(23)
                            ]
                            .padding(padding::top(3))
                            .spacing(8),
                            row![
                                svg("images/code.svg").width(23).height(23),
                                svg("images/text-quote.svg").width(23).height(23),
                                svg("images/link.svg").width(19).height(19),
                                svg("images/image.svg").width(19).height(19),
                                svg("images/at-sign.svg").width(19).height(19)
                            ]
                            .padding(padding::top(3))
                            .spacing(8),
                        ]
                        .spacing(20)
                    )
                    .align_right(Length::Fill)
                ]
                .padding(Padding {
                    top: 8.0,
                    right: 10.0,
                    bottom: 4.0,
                    left: 10.0
                })
            )
            .style(|_| theme.stylesheet.message_area_bar),
            text_editor(message_area_content)
                .padding(8)
                .height(*message_area_height)
                .on_action(move |action| Message::MessageAreaEdit(action))
                .placeholder("Type your message...")
                .style(|_, _| theme.stylesheet.message_area),
            row![
                row![
                    svg("images/smile.svg").width(20).height(20),
                    svg("images/upload.svg").width(20).height(20),
                ]
                .spacing(8),
                container(
                    mouse_area(
                        container(text("Send"))
                            .style(|_| theme.stylesheet.primary_button)
                            .padding(4)
                            .align_y(Alignment::Center)
                    )
                    .on_release(Message::PostMessage)
                )
                .align_right(Length::Fill)
            ]
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 8.0,
                left: 10.0
            })
        ])
        .style(|_| theme.stylesheet.message_area_container),
    )
    .padding(padding::top(10))
    .into()
}
