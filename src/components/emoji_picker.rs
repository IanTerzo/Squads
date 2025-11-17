use std::collections::HashMap;

use crate::style::Theme;
use crate::types::EmojiPickerAction;
use crate::widgets::gif::{self, Gif};
use crate::{utils, Message};
use iced::widget::{
    column, container, image, mouse_area, row, scrollable, svg, text, text_input, Space,
};
use iced::{Border, Color, Element, Font, Length, Padding};

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
    emoji_map: &'a HashMap<String, String>,
) -> Element<'a, Message> {
    let mut emoji_row = row![];
    for (i, (_emoji_id, emoji)) in emoji_map.iter().enumerate() {
        emoji_row = emoji_row.push(
            mouse_area(container(container(text(emoji).size(34)).width(38).height(38)).padding(2))
                .on_release(Message::EmojiPickerPicked(
                    _emoji_id.to_string(),
                    emoji.clone(),
                )),
        );
    }
    let emoji_scrollable = container(
        scrollable(column![text("People"), emoji_row.wrap()])
            .height(400)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(theme.features.scrollbar_width)
                    .spacing(theme.features.scrollable_spacing)
                    .scroller_width(theme.features.scrollbar_width),
            ))
            .width(420)
            .style(|_, _| theme.stylesheet.scrollable),
    )
    .padding(5);

    let categories = container(
        column![
            svg(utils::get_image_dir().join("clock.svg"))
                .width(24)
                .height(24),
            mouse_area(
                svg(utils::get_image_dir().join("smile.svg"))
                    .width(24)
                    .height(24)
            ),
            svg(utils::get_image_dir().join("leaf.svg"))
                .width(24)
                .height(24),
            svg(utils::get_image_dir().join("pizza.svg"))
                .width(24)
                .height(24),
            svg(utils::get_image_dir().join("gamepad-2.svg"))
                .width(24)
                .height(24),
            svg(utils::get_image_dir().join("bike.svg"))
                .width(24)
                .height(24),
            svg(utils::get_image_dir().join("lamp.svg"))
                .width(24)
                .height(24),
            svg(utils::get_image_dir().join("heart.svg"))
                .width(24)
                .height(24),
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

    let top_part =
        container(text_input("Find your emoji...", "").style(|_, _| theme.stylesheet.input))
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
        .on_release(Message::ToggleEmojiPicker(None, EmojiPickerAction::None))
        .into()
}
