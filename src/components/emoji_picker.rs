use crate::components::horizontal_line::c_horizontal_line;
use crate::style::Theme;
use crate::types::Emoji;
use crate::{Message, utils};
use iced::widget::{Id, column, container, mouse_area, row, scrollable, svg, text, text_input};
use iced::{Border, Element, Length, Padding};
use indexmap::IndexMap;

pub fn c_emoji_picker<'a, F>(
    theme: &'a Theme,
    search_emojis_input_value: &String,
    emoji_map: &'a IndexMap<String, Emoji>,
    on_pick: F,
) -> Element<'a, Message>
where
    F: Fn(String, String) -> Message + 'a,
{
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
                .interaction(iced::mouse::Interaction::Pointer)
                .on_release(on_pick(_emoji_id.to_string(), emoji.unicode.clone()));

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
                    .interaction(iced::mouse::Interaction::Pointer)
                    .on_release(on_pick(_emoji_id.to_string(), emoji.unicode.clone()));

                    emoji_row = emoji_row.push(emoji_component);
                }
            }
            column![emoji_row.wrap()]
        })
        .height(400)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(4)
                .spacing(0)
                .scroller_width(4),
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
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.0)),
            mouse_area(
                svg(utils::get_image_dir().join("smile.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.0)),
            mouse_area(
                svg(utils::get_image_dir().join("user-round.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.1261)),
            mouse_area(
                svg(utils::get_image_dir().join("leaf.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.342)),
            mouse_area(
                svg(utils::get_image_dir().join("pizza.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.45)),
            mouse_area(
                svg(utils::get_image_dir().join("gamepad-2.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.53)),
            mouse_area(
                svg(utils::get_image_dir().join("bike.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.59)),
            mouse_area(
                svg(utils::get_image_dir().join("lamp.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.734)),
            mouse_area(
                svg(utils::get_image_dir().join("heart.svg"))
                    .width(24)
                    .height(24)
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_release(Message::EmojiPickerScrollTo(0.9)),
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
        background: Some(theme.colors.background.into()),

        ..Default::default()
    });

    let top_part = container(
        text_input("Find your emoji...", search_emojis_input_value)
            .id(Id::new("search_emojis_input"))
            .padding(8)
            .on_input(Message::SearchEmojisContentChanged)
            .style(|_, _| text_input::Style {
                background: theme.colors.background.into(),
                border: Border {
                    color: theme.colors.line,
                    width: 1.0,
                    radius: 6.into(),
                },
                icon: theme.colors.not_set,
                placeholder: theme.colors.demo_text,
                value: theme.colors.text,
                selection: theme.colors.text_selection,
            }),
    )
    .padding(12);

    mouse_area(
        container(column![
            top_part,
            c_horizontal_line(theme, Length::Fill),
            row![categories, emoji_scrollable]
        ])
        .width(440)
        .height(400)
        .padding(2)
        .style(|_| container::Style {
            background: Some(theme.colors.foreground_alt.into()),
            border: Border {
                color: theme.colors.line,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }),
    )
    .on_enter(Message::EnterEmojiPicker)
    .on_exit(Message::ExitEmojiPicker)
    .interaction(iced::mouse::Interaction::Idle)
    .into()
}
