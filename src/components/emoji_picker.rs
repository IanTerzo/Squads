use std::collections::HashMap;

use crate::style::Theme;
use crate::types::EmojiPickerAction;
use crate::widgets::gif::{self, Gif};
use crate::Message;
use iced::widget::{column, container, image, mouse_area, row, scrollable, text, text_input};
use iced::{Color, Element, Font, Length, Padding};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EmojiPickerAlignment {
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct EmojiPickerPosition {
    pub alignment: EmojiPickerAlignment,
    pub padding: Padding,
}

pub fn c_emoji_picker<'a>(
    theme: &'a Theme,
    pos: &EmojiPickerPosition,
    emoji_map: &'a HashMap<String, String>,
) -> Element<'a, Message> {
    let mut emojies_column = column![];
    let mut emoji_row = row![];
    for (i, (_emoji_id, emoji)) in emoji_map.iter().enumerate() {
        emoji_row = emoji_row.push(
            mouse_area(container(container(text(emoji).size(28)).width(36).height(36)).padding(2))
                .on_release(Message::EmojiPickerPicked(
                    _emoji_id.to_string(),
                    emoji.clone(),
                )),
        );
        if (i + 1) % 9 == 0 {
            emojies_column = emojies_column.push(emoji_row);
            emoji_row = row![];
        }
    }
    let emoji_scrollable = scrollable(emojies_column).height(400);

    let categories = scrollable(column![text("a"), text("a"), text("a")]);

    let mut picker_screen = container(
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
    .padding(pos.padding);

    if pos.alignment == EmojiPickerAlignment::Bottom {
        picker_screen = picker_screen.align_bottom(Length::Fill);
    }

    mouse_area(picker_screen)
        .on_release(Message::ToggleEmojiPicker(None, EmojiPickerAction::None))
        .into()
}
