use std::collections::HashMap;

use crate::style::Theme;
use crate::types::{Emoji, EmojiPickerAction};
use crate::widgets::gif::{self, Gif};
use crate::{utils, Message};
use iced::widget::scrollable::Id;
use iced::widget::{
    column, container, image, mouse_area, row, scrollable, svg, text, text_input, Space,
};
use iced::{Border, Color, Element, Font, Length, Padding};
use indexmap::IndexMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EmojiPickerAlignment {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone)]
pub struct EmojiPickerPosition {
    pub alignment: EmojiPickerAlignment,
    pub padding: Padding,
}

pub fn c_emoji_picker<'a>(
    theme: &'a Theme,
    pos: EmojiPickerPosition,
    search_emojis_input_value: String,
    emoji_map: &'a IndexMap<String, Emoji>,
) -> Element<'a, Message> {
    let emoji_scrollable = container(
        scrollable(if search_emojis_input_value == "" {
            let mut emoji_cat1 = row![];
            let mut emoji_cat2 = row![];
            let mut emoji_cat3 = row![];
            let mut emoji_cat4 = row![];
            let mut emoji_cat5 = row![];
            let mut emoji_cat6 = row![];
            let mut emoji_cat7 = row![];
            let mut emoji_cat8 = row![];

            for (_emoji_id, emoji) in emoji_map {
                let emoji_component = mouse_area(
                    container(
                        container(text(emoji.unicode.clone()).size(34))
                            .width(38)
                            .height(38),
                    )
                    .padding(2),
                )
                .on_press(Message::EmojiPickerPicked(
                    _emoji_id.to_string(),
                    emoji.unicode.clone(),
                ));

                match emoji.category.as_str() {
                    "People & Body" => emoji_cat1 = emoji_cat1.push(emoji_component),
                    "Animals & Nature" => emoji_cat2 = emoji_cat2.push(emoji_component),
                    "Symbols" => emoji_cat3 = emoji_cat3.push(emoji_component),
                    "Smileys & Emotion" => emoji_cat4 = emoji_cat4.push(emoji_component),
                    "Objects" => emoji_cat5 = emoji_cat5.push(emoji_component),
                    "Food & Drink" => emoji_cat6 = emoji_cat6.push(emoji_component),
                    "Travel & Places" => emoji_cat7 = emoji_cat7.push(emoji_component),
                    _ => emoji_cat8 = emoji_cat8.push(emoji_component),
                };
            }

            column![
                text("Smileys & Emotion"),
                emoji_cat4.wrap(),
                text("People & Body"),
                emoji_cat1.wrap(),
                text("Animals & Nature"),
                emoji_cat2.wrap(),
                text("Food & Drink"),
                emoji_cat6.wrap(),
                text("Activities"),
                emoji_cat8.wrap(),
                text("Travel & Places"),
                emoji_cat7.wrap(),
                text("Objects"),
                emoji_cat5.wrap(),
                text("Symbols"),
                emoji_cat3.wrap(),
            ]
        } else {
            let mut emoji_row = row![];

            for (_emoji_id, emoji) in emoji_map {
                if emoji.keywords.iter().any(|s| {
                    s.to_lowercase()
                        .contains(search_emojis_input_value.to_lowercase().as_str())
                }) {
                    let emoji_component = mouse_area(
                        container(
                            container(text(emoji.unicode.clone()).size(34))
                                .width(38)
                                .height(38),
                        )
                        .padding(2),
                    )
                    .on_press(Message::EmojiPickerPicked(
                        _emoji_id.to_string(),
                        emoji.unicode.clone(),
                    ));

                    emoji_row = emoji_row.push(emoji_component);
                }
            }
            column![emoji_row.wrap()]
        })
        .height(400)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(theme.features.scrollbar_width)
                .spacing(theme.features.scrollable_spacing)
                .scroller_width(theme.features.scrollbar_width),
        ))
        .width(420)
        .id(Id::new("emoji_column"))
        .style(|_, _| theme.stylesheet.scrollable),
    )
    .padding(5);

    let categories = container(
        column![
            mouse_area(
                svg(utils::get_image_dir().join("clock.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.0)),
            mouse_area(
                svg(utils::get_image_dir().join("smile.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.0)),
            mouse_area(
                svg(utils::get_image_dir().join("user-round.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.1261)),
            mouse_area(
                svg(utils::get_image_dir().join("leaf.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.342)),
            mouse_area(
                svg(utils::get_image_dir().join("pizza.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.45)),
            mouse_area(
                svg(utils::get_image_dir().join("gamepad-2.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.53)),
            mouse_area(
                svg(utils::get_image_dir().join("bike.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.59)),
            mouse_area(
                svg(utils::get_image_dir().join("lamp.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.734)),
            mouse_area(
                svg(utils::get_image_dir().join("heart.svg"))
                    .width(24)
                    .height(24)
            )
            .on_press(Message::EmojiPickerScrollTo(0.9)),
        ]
        .height(Length::Fill)
        .spacing(8),
    )
    .padding(Padding {
        top: 5.0,
        right: 9.0,
        left: 9.0,
        bottom: 6.0,
    })
    .style(|_| container::Style {
        background: Some(theme.colors.primary1.into()),

        ..Default::default()
    });

    let top_part = container(
        text_input("Find your emoji...", &search_emojis_input_value)
            .on_input(Message::SearchEmojisContentChanged)
            .style(|_, _| theme.stylesheet.input),
    )
    .padding(12);

    let mut picker_screen = container(
        container(column![
            top_part,
            container(Space::new(Length::Fill, 1)).style(|_| container::Style {
                background: Some(theme.colors.primary3.into()),
                ..Default::default()
            }),
            row![categories, emoji_scrollable]
        ])
        .width(440)
        .height(400)
        .padding(2)
        .style(|_| container::Style {
            background: Some(theme.colors.primary2.into()),
            border: Border {
                color: theme.colors.secondary1,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(pos.padding);

    match pos.alignment {
        EmojiPickerAlignment::TopLeft => {}
        EmojiPickerAlignment::BottomLeft => {
            picker_screen = picker_screen.align_bottom(Length::Fill)
        }
        EmojiPickerAlignment::TopRight => picker_screen = picker_screen.align_right(Length::Fill),
        EmojiPickerAlignment::BottomRight => {
            picker_screen = picker_screen.align_bottom(Length::Fill);
            picker_screen = picker_screen.align_right(Length::Fill)
        }
    }

    mouse_area(picker_screen)
        .on_press(Message::ToggleEmojiPicker(None, EmojiPickerAction::None))
        .into()
}
