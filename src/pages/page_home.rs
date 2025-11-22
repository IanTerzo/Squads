use std::collections::HashMap;

use crate::api;
use crate::api::Profile;
use crate::api::Team;
use crate::components::conversation::c_conversation;
use crate::style;
use crate::types::Emoji;
use crate::websockets::Presence;
use crate::Message;

use iced::widget::mouse_area;
use iced::widget::Space;
use iced::widget::{column, container, row, scrollable, text, text_input, Column, MouseArea};
use iced::Length;
use iced::Padding;
use iced::{padding, Alignment, Element};

use crate::components::{cached_image::c_cached_image, preview_message::c_preview_message};
use crate::utils::truncate_name;

pub fn home<'a>(
    theme: &'a style::Theme,
    teams: &Vec<Team>,
    activities: &Vec<crate::api::Message>,
    expanded_conversations: HashMap<String, (bool, Vec<api::Message>)>,
    emoji_map: &'a HashMap<String, Emoji>,
    users: &HashMap<String, Profile>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
    window_width: f32,
    search_teams_input_value: String,
) -> Element<'a, Message> {
    let mut teams_column: Column<Message> =
        column![]
            .spacing(theme.features.list_spacing)
            .padding(Padding {
                right: 4.0,
                left: 6.0,
                top: 6.0,
                bottom: 6.0,
            });

    for team in teams {
        if !team
            .display_name
            .to_lowercase()
            .starts_with(&search_teams_input_value.to_lowercase())
        {
            continue;
        }

        let team_picture = c_cached_image(
            team.picture_e_tag
                .clone()
                .unwrap_or(team.display_name.clone()),
            Message::FetchTeamImage(
                team.picture_e_tag
                    .clone()
                    .unwrap_or(team.display_name.clone()),
                team.picture_e_tag.clone().unwrap_or("".to_string()),
                team.team_site_information.group_id.clone(),
                team.display_name.clone(),
            ),
            28.0,
            28.0,
        );

        teams_column = teams_column.push(
            MouseArea::new(
                container(
                    row![
                        container(team_picture).padding(padding::left(11)),
                        text(truncate_name(team.display_name.clone(), 16)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                )
                .style(|_| theme.stylesheet.list_tab)
                .center_y(47)
                .width(220),
            )
            .on_release(Message::OpenTeam(team.id.clone(), team.id.clone()))
            .on_enter(Message::PrefetchTeam(team.id.clone(), team.id.clone())),
        );
    }

    let team_scrollbar = container(
        scrollable(teams_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(theme.features.scrollbar_width)
                    .spacing(theme.features.scrollable_spacing)
                    .scroller_width(theme.features.scrollbar_width),
            ))
            .style(|_, _| theme.stylesheet.side_scrollable),
    );

    let search_teams = container(
        text_input("Search teams...", &search_teams_input_value)
            .on_input(Message::SearchTeamsContentChanged)
            .padding(6)
            .style(|_, _| theme.stylesheet.input),
    )
    .width(234)
    .padding(Padding {
        top: 8.0,
        left: 7.0,
        bottom: 7.0,
        right: 7.0,
    });

    let side_panel = container(column![
        search_teams,
        container(
            container(Space::new(Length::Fill, 1)).style(|_| container::Style {
                background: Some(theme.colors.primary3.into()),
                ..Default::default()
            })
        )
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 8.0,
            right: 8.0
        }),
        container(Space::new(Length::Fill, 2)).style(|_| container::Style {
            background: Some(theme.colors.primary1.into()),
            ..Default::default()
        }),
        team_scrollbar
    ])
    .style(|_| container::Style {
        background: Some(theme.colors.primary1.into()),
        ..Default::default()
    })
    .width(230)
    .height(Length::Fill);

    let mut activities_colum = column![].spacing(12).padding(Padding {
        left: 8.0,
        right: 8.0,
        top: 0.0,
        bottom: 0.0,
    });

    let activities_conversations: Vec<_> = activities.iter().rev().cloned().collect();

    for message in activities_conversations {
        if let Some(activity) = message.properties.clone().unwrap().activity {
            let thread_id = activity.source_thread_id.clone();

            let message_id = activity
                .source_reply_chain_id
                .unwrap_or(activity.source_message_id);

            let message_activity_id = message.id.unwrap().to_string();

            if let Some(conversation) = expanded_conversations.get(&message_activity_id) {
                if conversation.0 {
                    if conversation.1.len() > 0 {
                        let message = c_conversation(
                            theme,
                            conversation.1.iter().rev().cloned().collect(), // Can be optimized,
                            thread_id.clone(),
                            message_activity_id.clone(),
                            false,
                            emoji_map,
                            users,
                            me,
                            user_presences,
                        );
                        if let Some(message) = message {
                            activities_colum = activities_colum.push(
                                mouse_area(message).on_release(Message::ToggleExpandActivity(
                                    thread_id,
                                    message_id,
                                    message_activity_id,
                                )),
                            );
                        }
                    } else {
                        activities_colum = activities_colum.push(
                            mouse_area(text("Failed to load conversation.")).on_release(
                                Message::ToggleExpandActivity(
                                    thread_id,
                                    message_id,
                                    message_activity_id,
                                ),
                            ),
                        );
                    }
                } else {
                    activities_colum = activities_colum.push(
                        mouse_area(c_preview_message(theme, activity, window_width, emoji_map))
                            .on_release(Message::ToggleExpandActivity(
                                thread_id,
                                message_id,
                                message_activity_id,
                            )),
                    );
                }
            } else {
                activities_colum = activities_colum.push(
                    mouse_area(c_preview_message(theme, activity, window_width, emoji_map))
                        .on_release(Message::ToggleExpandActivity(
                            thread_id,
                            message_id,
                            message_activity_id,
                        )),
                );
            }
        }
    }

    let activities_scrollbar = container(
        scrollable(activities_colum)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(theme.features.scrollbar_width)
                    .spacing(theme.features.scrollable_spacing)
                    .scroller_width(theme.features.scrollbar_width),
            ))
            .anchor_bottom()
            .style(|_, _| theme.stylesheet.scrollable),
    )
    .padding(Padding {
        top: 8.0,
        right: 3.0,
        left: 0.0,
        bottom: 0.0,
    })
    .height(Length::Fill);

    row![side_panel, activities_scrollbar]
        .spacing(theme.features.page_row_spacing)
        .into()
}
