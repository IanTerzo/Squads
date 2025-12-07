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
use iced::widget::{column, container, scrollable, text};
use iced::Element;
use iced::Length;
use iced::Padding;
use indexmap::IndexMap;

use crate::components::preview_message::c_preview_message;

pub fn home<'a>(
    theme: &'a style::Theme,
    _teams: &Vec<Team>,
    activities: &Vec<crate::api::Message>,
    expanded_conversations: HashMap<String, (bool, Vec<api::Message>)>,
    emoji_map: &'a IndexMap<String, Emoji>,
    users: &HashMap<String, Profile>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
    window_width: f32,
    _search_teams_input_value: String,
) -> Element<'a, Message> {
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

    activities_scrollbar.into()
}
