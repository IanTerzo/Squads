use std::collections::HashMap;

use crate::style::Theme;
use crate::widgets::gif::{self, Gif};
use crate::Message;
use iced::widget::{column, container, image, mouse_area, row, scrollable, text, text_input};
use iced::{Color, Element, Font, Length, Padding};

pub fn c_emoji_picker<'a>(
    theme: &'a Theme,
    emoji_map: &'a HashMap<String, String>,
) -> Element<'a, Message> {
    let mut emojies_column = column![];
    let mut emoji_row = row![];
    for (i, (_emoji_id, emoji)) in emoji_map.iter().enumerate() {
        emoji_row = emoji_row.push(
            mouse_area(
                container(
                    container(text(emoji).font(Font::with_name("Twemoji")).size(28))
                        .width(36)
                        .height(36),
                )
                .padding(2),
            )
            .on_release(Message::Hello(_emoji_id.to_string())),
        );
        if (i + 1) % 9 == 0 {
            emojies_column = emojies_column.push(emoji_row);
            emoji_row = row![];
        }
    }
    let emoji_scrollable = scrollable(emojies_column).height(400);

    let categories = scrollable(column![text("a"), text("a"), text("a")]);
    mouse_area(
        container(
            container(column![
                text_input("Find your emoji...", ""),
                row![categories, emoji_scrollable]
            ])
            .width(364)
            .style(|_| container::Style {
                background: Some(theme.colors.secondary1.into()),
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(Padding {
            bottom: 150.0,
            left: 260.0,
            top: 0.0,
            right: 0.0,
        }),
    )
    .on_release(Message::ToggleEmojiPicker)
    .into()
}
