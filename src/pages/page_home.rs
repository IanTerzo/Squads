use std::collections::HashMap;

use crate::api;
use crate::api::Conversation;
use crate::api::Profile;
use crate::api::Team;
use crate::components::conversation::c_conversation;
use crate::style;
use crate::Message;

use iced::widget::mouse_area;
use iced::widget::{column, container, row, scrollable, text, text_input, Column, MouseArea};
use iced::Length;
use iced::{padding, Alignment, Element};

use crate::components::{cached_image::c_cached_image, preview_message::c_preview_message};
use crate::utils::truncate_name;

pub fn home<'a>(
    theme: &'a style::Theme,
    teams: &Vec<Team>,
    activities: &Vec<crate::api::Message>,
    expanded_conversations: HashMap<String, Vec<api::Message>>,
    emoji_map: &'a HashMap<String, String>,
    users: &HashMap<String, Profile>,
    window_width: f32,
    search_teams_input_value: String,
) -> Element<'a, Message> {
    let mut teams_column: Column<Message> = column![].spacing(theme.features.list_spacing);

    let mut teams_list_empty = true;

    for team in teams {
        if !team
            .display_name
            .to_lowercase()
            .starts_with(&search_teams_input_value.to_lowercase())
        {
            continue;
        }

        teams_list_empty = false;

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
                        container(team_picture).padding(padding::left(10)),
                        text(truncate_name(team.display_name.clone(), 16)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                )
                .style(|_| theme.stylesheet.list_tab)
                .center_y(47)
                .width(220),
            )
            .on_press(Message::OpenTeam(team.id.clone(), team.id.clone()))
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
            .style(|_, _| theme.stylesheet.scrollable),
    );

    let search_teams = container(
        text_input("Search teams...", &search_teams_input_value)
            .on_input(Message::SearchTeamsContentChanged)
            .padding(8)
            .style(|_, _| theme.stylesheet.input),
    )
    .width(220)
    .padding(padding::bottom(18));

    let mut side_panel = column![search_teams, team_scrollbar];

    // Mantain the same padding as the scrollbar
    if teams_list_empty {
        side_panel = side_panel.padding(padding::right(19));
    }

    let mut activities_colum = column![].spacing(12);
    let activities_conversations: Vec<_> = activities.iter().rev().cloned().collect();

    for message in activities_conversations {
        if let Some(activity) = message.properties.clone().unwrap().activity
        {
            let thread_id = activity.source_thread_id.clone();
    
            let message_id = activity
                .source_reply_chain_id
                .unwrap_or(activity.source_message_id);
    
            let message_activity_id = message.id.unwrap().to_string();
    
            if let Some(conversation) = expanded_conversations.get(&message_activity_id) {
                if conversation.len() > 0 {
                    let message = c_conversation(
                        theme,
                        conversation.iter().rev().cloned().collect(), // Can be optimized
                        message_activity_id.clone(),
                        false,
                        emoji_map,
                        users,
                    );
                    if let Some(message) = message {
                        activities_colum = activities_colum.push(mouse_area(message).on_release(
                            Message::ExpandActivity(thread_id, message_id, message_activity_id),
                        ));
                    }
                } else {
                    activities_colum = activities_colum.push(
                        mouse_area(text("Failed to load conversation.")).on_release(
                            Message::ExpandActivity(thread_id, message_id, message_activity_id),
                        ),
                    );
                }
            } else {
                activities_colum = activities_colum.push(
                    mouse_area(c_preview_message(theme, activity, window_width, emoji_map)).on_release(
                        Message::ExpandActivity(thread_id, message_id, message_activity_id),
                    ),
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
    .height(Length::Fill);

    row![side_panel, activities_scrollbar]
        .spacing(theme.features.page_row_spacing)
        .into()
}
