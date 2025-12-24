use std::collections::HashMap;

use crate::{
    Message,
    api::{Chat, Profile},
    style::Theme,
};
use iced::{
    Border, Element, Length, Padding,
    alignment::Vertical,
    border,
    widget::{Id, checkbox, column, container, mouse_area, row, scrollable, text, text_input},
};

pub fn c_add_users<'a>(
    theme: &'a Theme,
    current_chat: &Chat,
    users: &'a HashMap<String, Profile>,
    me: &'a Profile,
    add_users_cheked: &HashMap<String, bool>,
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

        // Do not show user if already part of group or chat
        if current_chat
            .members
            .iter()
            .any(|member| &member.mri.replace("8:orgid:", "") == user.0)
        {
            continue;
        }

        // Remove yourself if the alst check din't chatch you if in start chat body
        if *user.0 == me.id {
            continue;
        }

        let is_checked = add_users_cheked.get(user.0).unwrap_or(&false).clone();
        let profile_row = row![
            text(
                user.1
                    .display_name
                    .clone()
                    .unwrap_or("Unknown User".to_string())
            ),
            container(checkbox(is_checked).style(|_, _| checkbox::Style {
                background: theme.colors.foreground_surface.into(),
                border: border::rounded(2),
                icon_color: theme.colors.text,
                text_color: None,
            }))
            .align_right(Length::Fill)
        ]
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
                .interaction(iced::mouse::Interaction::Pointer)
                .on_release(Message::ToggleUserCheckbox(is_checked, user.0.to_string())),
            )
            .into();
    }

    mouse_area(
        container(
            column![
                row![
                    text_input("Search users...", &search_users_input_value)
                        .on_input(Message::SearchUsersContentChanged)
                        .padding(10)
                        .id("search_users_input")
                        .style(|_, _| theme.stylesheet.input),
                    mouse_area(
                        container(text("Add"))
                            .padding(Padding {
                                top: 8.0,
                                bottom: 8.0,
                                left: 13.0,
                                right: 13.0
                            })
                            .style(|_| {
                                container::Style {
                                    background: Some(theme.colors.foreground_surface.into()),
                                    border: border::rounded(4),
                                    ..Default::default()
                                }
                            })
                    )
                    .interaction(iced::mouse::Interaction::Pointer)
                    .on_release(
                        if !current_chat.is_one_on_one.unwrap_or(false) {
                            Message::AddToGroupChat(
                                current_chat.id.clone(),
                                add_users_cheked
                                    .iter()
                                    .filter_map(
                                        |(key, &val)| if val { Some(key.clone()) } else { None },
                                    )
                                    .collect(),
                            )
                        } else {
                            if add_users_cheked.len() > 0 {
                                Message::StartChat({
                                    let mut users: Vec<String> = add_users_cheked
                                        .iter()
                                        .filter_map(
                                            |(key, &val)| {
                                                if val { Some(key.clone()) } else { None }
                                            },
                                        )
                                        .collect();

                                    users.push(
                                        current_chat
                                            .members
                                            .iter()
                                            .find(|member| {
                                                &member.mri.replace("8:orgid:", "") != &me.id
                                            })
                                            .unwrap()
                                            .mri
                                            .clone()
                                            .replace("8:orgid:", ""),
                                    );

                                    users
                                })
                            } else {
                                Message::DoNothing(())
                            }
                        }
                    )
                ]
                .align_y(Vertical::Center)
                .spacing(10)
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
