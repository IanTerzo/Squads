use std::collections::HashMap;

use crate::{Message, api::Profile, style::Theme};
use iced::{
    Border, Element, Length, Padding,
    alignment::Vertical,
    border,
    widget::{Id, column, container, mouse_area, scrollable, text, text_input},
};

pub fn c_start_chat<'a>(
    theme: &'a Theme,
    users: &'a HashMap<String, Profile>,
    me: &'a Profile,
    search_users_input_value: &String,
) -> Element<'a, Message> {
    let mut user_column = column![]
        .spacing(theme.features.list_spacing)
        .padding(Padding {
            left: 8.0,
            right: 6.0,
            top: 6.0,
            bottom: 6.0,
        });

    for user in users {
        // Hotfix, this removes all non "human" users
        if user.1.surname.is_none() || user.1.display_name.is_none() {
            continue;
        }

        if !user
            .1
            .display_name
            .as_ref()
            .unwrap()
            .to_lowercase()
            .starts_with(&search_users_input_value.to_lowercase())
        {
            continue;
        }

        // Remove yourself if the alst check din't chatch you if in start chat body
        if *user.0 == me.id {
            continue;
        }

        let profile_row = container(text(
            user.1
                .display_name
                .clone()
                .unwrap_or("Unknown User".to_string()),
        ))
        .width(Length::Fill)
        .align_y(Vertical::Center);

        user_column = user_column
            .push(
                mouse_area(
                    container(profile_row)
                        .style(|_| container::Style {
                            background: Some(theme.colors.foreground.into()),
                            border: border::rounded(4),
                            ..Default::default()
                        })
                        .padding(Padding {
                            top: 9.0,
                            right: 6.0,
                            bottom: 9.0,
                            left: 6.0,
                        }),
                )
                .on_release(Message::StartChat(vec![user.0.clone()])),
            )
            .into();
    }

    mouse_area(
        container(
            column![
                container(
                    text_input("Search users...", &search_users_input_value)
                        .on_input(Message::SearchUsersContentChanged)
                        .padding(10)
                        .id("search_users_input")
                        .style(|_, _| theme.stylesheet.input)
                )
                .align_y(Vertical::Center)
                .padding(Padding {
                    top: 6.0,
                    bottom: 0.0,
                    left: 8.0,
                    right: 8.0
                }),
                scrollable(user_column)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new()
                            .width(theme.features.scrollbar_width)
                            .spacing(theme.features.scrollable_spacing)
                            .scroller_width(theme.features.scrollbar_width),
                    ))
                    .style(|_, _| theme.stylesheet.scrollable)
                    .id(Id::new("members_column"))
            ]
            .spacing(6),
        )
        .width(450)
        .height(500)
        .padding(1) // Otherwise the border is bugging
        .style(|_| container::Style {
            background: Some(theme.colors.foreground.into()),
            border: Border {
                color: theme.colors.line,
                width: 1.0,
                radius: 4.into(),
            },
            ..Default::default()
        }),
    )
    .on_enter(Message::EnterCenteredOverlay)
    .on_exit(Message::ExitCenteredOverlay)
    .into()
}
