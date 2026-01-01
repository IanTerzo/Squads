use std::collections::HashMap;

use crate::Message;
use crate::api;
use crate::api::Profile;
use crate::components::conversation::c_conversation;
use crate::components::preview_message::c_preview_message;
use crate::style;
use crate::types::Emoji;
use crate::websockets::Presence;
use iced::Element;
use iced::Length;
use iced::Padding;
use iced::widget::container;
use iced::widget::mouse_area;
use iced::widget::{column, scrollable, text};
use indexmap::IndexMap;

pub fn activity<'a>(
    theme: &'a style::Theme,
    activities: &Vec<crate::api::Message>,
    expanded_conversations: &HashMap<String, (bool, Vec<api::Message>)>,
    search_emojis_input_value: &String,
    emoji_map: &'a IndexMap<String, Emoji>,
    users: &'a HashMap<String, Profile>,
    user_presences: &'a HashMap<String, Presence>,
    me: &'a Profile,
    show_plus_emoji_picker: &'a bool,
    emoji_picker_message_id: &'a Option<String>,
    window_size: &(f32, f32),
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
                            search_emojis_input_value,
                            users,
                            me,
                            user_presences,
                            show_plus_emoji_picker,
                            emoji_picker_message_id,
                            window_size,
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
                        mouse_area(c_preview_message(
                            theme,
                            activity,
                            &window_size.0,
                            emoji_map,
                        ))
                        .on_release(Message::ToggleExpandActivity(
                            thread_id,
                            message_id,
                            message_activity_id,
                        )),
                    );
                }
            } else {
                activities_colum = activities_colum.push(
                    mouse_area(c_preview_message(
                        theme,
                        activity,
                        &window_size.0,
                        emoji_map,
                    ))
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
                    .width(4)
                    .spacing(0)
                    .scroller_width(4),
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
