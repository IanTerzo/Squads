use std::collections::HashMap;
use std::str::FromStr;

use crate::Message;
use crate::api::{self, Chat, Profile};
use crate::components::conversation::c_conversation;
use crate::components::horizontal_line::c_horizontal_line;
use crate::components::picture_and_status::c_picture_and_status;
use crate::components::preview_message::c_preview_message;
use crate::components::toooltip::c_tooltip;
use crate::components::{
    cached_image::c_cached_image, chat_message::c_chat_message, message_area::c_message_area,
};
use crate::types::Emoji;
use crate::utils::{self, truncate_name};
use crate::websockets::Presence;
use crate::widgets::circle::circle;
use crate::{ChatBody, style};

use iced::alignment::Vertical;
use iced::task::Handle;
use iced::widget::text_editor::Content;
use iced::widget::{Id, column, container, mouse_area, row, space, svg, tooltip};
use iced::widget::{scrollable, text};
use iced::{Alignment, Color, Element, Length, Padding, border, padding};
use indexmap::IndexMap;

fn get_chat_title(chat: &Chat, user_id: &String, users: &HashMap<String, Profile>) -> String {
    if let Some(chat_title) = &chat.title {
        chat_title.clone()
    } else {
        // Filter out the current user
        let other_members: Vec<_> = chat
            .members
            .iter()
            .filter_map(|member| {
                let member_id = member.mri.replace("8:orgid:", "");
                if &member_id != user_id {
                    Some(member_id)
                } else {
                    None
                }
            })
            .collect();

        if other_members.len() == 1 {
            // One-on-one chat
            let member_id = &other_members[0];
            users
                .get(member_id)
                .and_then(|profile| profile.display_name.clone())
                .unwrap_or_else(|| "Unknown User".to_string())
        } else {
            // Group chat
            let names: Vec<_> = other_members
                .iter()
                .map(|member_id| {
                    users
                        .get(member_id)
                        .and_then(|profile| profile.display_name.clone())
                        .unwrap_or_else(|| "Unknown User".to_string())
                })
                .collect();

            names.join(", ")
        }
    }
}

fn get_chat_picture<'a>(
    chat: &'a Chat,
    user_id: &'a String,
    users: &'a HashMap<String, Profile>,
) -> Element<'a, Message> {
    if let Some(chat_picture) = &chat.picture {
        let url = chat_picture.strip_prefix("URL@").unwrap_or(chat_picture);
        let identifier = url.replace("https:", "").replace("/", "").replace(":", "");

        c_cached_image(
            identifier.clone(),
            Message::AuthorizeImage(url.to_string(), identifier),
            28.0,
            28.0,
            4.0,
        )
    } else {
        let member_profiles: Vec<_> = chat
            .members
            .iter()
            .filter_map(|member| {
                let member_id = member.mri.strip_prefix("8:orgid:").unwrap_or(&member.mri);
                if member_id != user_id {
                    users
                        .get(member_id)
                        .and_then(|profile| profile.display_name.clone())
                        .map(|name| (member.mri.clone(), name))
                } else {
                    None
                }
            })
            .collect();

        let identifier = member_profiles
            .iter()
            .take(3)
            .map(|(mri, _)| mri)
            .cloned()
            .collect::<Vec<_>>()
            .join("-")
            .replace(":", "");

        if identifier != "" {
            c_cached_image(
                identifier.clone(),
                Message::FetchMergedProfilePicture(identifier, member_profiles),
                28.0,
                28.0,
                4.0,
            )
        } else {
            container(container(space()))
                .style(|_| container::Style {
                    background: Some(
                        Color::from_str("#b8b4b4")
                            .expect("Background color is invalid.")
                            .into(),
                    ),

                    ..Default::default()
                })
                .width(28.0)
                .height(28.0)
                .into()
        }
    }
}

pub fn chat<'a>(
    theme: &'a style::Theme,
    current_chat: Option<&'a Chat>,
    users_typing: &HashMap<String, HashMap<String, Handle>>,
    chats: &'a Vec<Chat>,
    conversation: &Option<&Vec<api::Message>>,
    chat_message_options: &'a HashMap<String, bool>,
    chat_list_options: &'a HashMap<String, bool>,
    search_emojis_input_value: &String,
    emoji_map: &'a IndexMap<String, Emoji>,
    users: &'a HashMap<String, Profile>,
    user_presences: &'a HashMap<String, Presence>,
    me: &'a Profile,
    search_chats_input_value: String,
    message_area_content: &'a Content,
    message_area_height: &f32,
    activities: &Vec<crate::api::Message>,
    expanded_conversations: &HashMap<String, (bool, Vec<api::Message>)>,
    page_body: &'a ChatBody,
    show_more_options: &'a bool,
    more_menu_message_id: &'a Option<String>,
    show_message_area_emoji_picker: &bool,
    show_message_emoji_picker: &'a bool,
    show_plus_emoji_picker: &'a bool,
    emoji_picker_message_id: &'a Option<String>,
    window_size: &(f32, f32),
) -> Element<'a, Message> {
    let mut page = row![].spacing(theme.features.page_row_spacing);

    // Side panel

    let additionals = column![
        container(column![
            mouse_area(
                container(
                    row![
                        svg(utils::get_image_dir().join("bell.svg"))
                            .width(20)
                            .height(20),
                        text!("Activity")
                    ]
                    .spacing(8)
                )
                .width(190)
                .padding(Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 8.0,
                    right: 8.0,
                })
                .style(move |_| {
                    container::Style {
                        background: Some(theme.colors.foreground.into()),
                        border: border::rounded(6),
                        ..Default::default()
                    }
                })
            )
            .on_release(Message::ToggleActivity)
            .interaction(iced::mouse::Interaction::Pointer)
        ])
        .padding(Padding {
            top: 8.0,
            bottom: 14.0,
            left: 10.0,
            right: 0.0,
        }),
        container(c_horizontal_line(theme, 210.into()))
            .width(Length::Fill)
            .align_x(Alignment::Center),
        container(
            text("Direct Messages")
                .size(14)
                .color(theme.colors.demo_text)
        )
        .padding(Padding {
            top: 10.0,
            bottom: 2.0,
            right: 0.0,
            left: 8.0
        })
    ];

    let mut chats_column = column![additionals]
        .spacing(theme.features.list_spacing)
        .padding(Padding {
            right: 4.0,
            left: 6.0,
            top: 6.0,
            bottom: 6.0,
        });

    for chat in chats {
        let chat_title = get_chat_title(&chat, &me.id, &users);
        if !chat_title
            .to_lowercase()
            .starts_with(&search_chats_input_value.to_lowercase())
        {
            continue;
        }

        let mut chat_items = row![].align_y(Alignment::Center);

        if !chat.is_read.unwrap_or(true) {
            chat_items = chat_items.push(circle(2.5, theme.colors.notification))
        } else {
            chat_items = chat_items.push(space().width(5))
        }

        let picture = get_chat_picture(&chat, &me.id, &users);

        if chat.is_one_on_one.unwrap_or(false) {
            let presence = user_presences.get(
                &chat
                    .members
                    .iter()
                    .find(|member| member.mri != format!("8:orgid:{}", me.id))
                    .unwrap()
                    .mri,
            );

            chat_items = chat_items.push(
                container(c_picture_and_status(theme, picture, presence, (28.0, 28.0)))
                    .padding(padding::right(4)),
            );
        } else {
            chat_items = chat_items.push(container(picture).padding(Padding {
                left: 6.0,
                right: 10.0,
                top: 6.0,
                bottom: 6.0,
            }));
        }

        let mut chat_info_column = column![text(truncate_name(chat_title, 20))];

        if users_typing.get(&chat.id).map_or(false, |u| !u.is_empty()) {
            chat_info_column =
                chat_info_column.push(text("is typing...").size(14).color(theme.colors.demo_text));
        } else if chat.chat_type.clone().unwrap_or("any".to_string()) == "draft" {
            chat_info_column =
                chat_info_column.push(text("Draft").size(14).color(theme.colors.demo_text));
        } else if !chat.is_one_on_one.unwrap_or(true) {
            chat_info_column = chat_info_column.push(
                text!("{} members", chat.members.len())
                    .size(14)
                    .color(theme.colors.demo_text),
            );
        }

        chat_items = chat_items.push(chat_info_column);

        let chat_item = mouse_area(
            container(chat_items)
                .style(move |_| {
                    if let Some(current_chat) = current_chat {
                        if chat.id == current_chat.id && *page_body != ChatBody::Activity {
                            container::Style {
                                background: Some(
                                    theme.colors.foreground_button_nobg_selected.into(),
                                ),
                                border: border::rounded(6),
                                ..Default::default()
                            }
                        } else if chat_list_options.get(&chat.id).unwrap_or(&false).to_owned() {
                            container::Style {
                                background: Some(
                                    theme.colors.foreground_button_nobg_hovered.into(),
                                ),
                                border: border::rounded(6),
                                ..Default::default()
                            }
                        } else {
                            container::Style {
                                background: Some(theme.colors.foreground.into()),
                                border: border::rounded(6),
                                ..Default::default()
                            }
                        }
                    } else {
                        container::Style {
                            background: Some(theme.colors.foreground.into()),
                            border: border::rounded(6),
                            ..Default::default()
                        }
                    }
                })
                .center_y(47)
                .width(220),
        )
        .on_enter(Message::PrefetchChat(chat.id.clone()))
        .on_exit(Message::StopShowChatListOptions(chat.id.clone()))
        .on_release(Message::OpenChat(chat.id.clone()))
        .interaction(iced::mouse::Interaction::Pointer);

        chats_column = chats_column.push(chat_item);
    }

    let chats_scrollable = scrollable(chats_column)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(theme.features.scrollbar_width)
                .spacing(theme.features.scrollable_spacing)
                .scroller_width(theme.features.scrollbar_width),
        ))
        .style(|_, _| theme.stylesheet.scrollable);

    let chat_options = column![
        container(
            container(
                mouse_area(text("Start or find a chat"))
                    .interaction(iced::mouse::Interaction::Pointer)
                    .on_release(Message::ToggleNewChatMenu)
            )
            .padding(Padding {
                top: 5.0,
                bottom: 5.0,
                left: 30.0,
                right: 30.0
            })
            .style(|_| container::Style {
                background: Some(theme.colors.foreground_button.into()),
                border: border::rounded(4),
                ..Default::default()
            })
        )
        .height(48)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Vertical::Center),
        c_horizontal_line(theme, Length::Fill),
        container(space().width(Length::Fill).height(2)).style(|_| container::Style {
            background: Some(theme.colors.foreground.into()),
            ..Default::default()
        })
    ];

    let side_panel = container(column![chat_options, chats_scrollable].width(230))
        .style(|_| container::Style {
            background: Some(theme.colors.foreground.into()),
            ..Default::default()
        })
        .height(Length::Fill);

    page = page.push(side_panel);

    // Chat page body

    if let Some(current_chat) = current_chat {
        let title = truncate_name(get_chat_title(&current_chat, &me.id, &users), 52);
        let picture = get_chat_picture(&current_chat, &me.id, &users);

        // Chat page title

        let title_row = row![
            space().width(2),
            picture,
            space().width(15),
            text(title),
            if let Some(is_one_on_one) = current_chat.is_one_on_one {
                if !is_one_on_one {
                    row![
                        space().width(8),
                        svg(utils::get_image_dir().join("pencil.svg"))
                            .width(17)
                            .height(17),
                    ]
                } else {
                    row![]
                }
            } else {
                row![]
            },
            container(
                row![
                    tooltip(
                        mouse_area(
                            svg(utils::get_image_dir().join("users-round.svg"))
                                .width(19)
                                .height(19)
                        )
                        .on_release(Message::ToggleShowChatMembers)
                        .interaction(iced::mouse::Interaction::Pointer),
                        c_tooltip(theme, "Show Member List"),
                        tooltip::Position::Bottom
                    ),
                    tooltip(
                        mouse_area(
                            svg(utils::get_image_dir().join("user-round-plus.svg"))
                                .width(19)
                                .height(19)
                        )
                        .on_release(Message::ToggleShowChatAdd)
                        .interaction(iced::mouse::Interaction::Pointer),
                        c_tooltip(theme, "Add Users to DM"),
                        tooltip::Position::Bottom
                    ),
                ]
                .spacing(14)
            )
            .align_right(Length::Fill)
        ]
        .padding(Padding {
            top: 10.0,
            right: 30.0,
            bottom: 10.0,
            left: 14.0,
        })
        .align_y(Alignment::Center);

        let title_row_container = column![
            container(title_row).style(|_| container::Style {
                background: Some(theme.colors.background.into()),
                ..Default::default()
            }),
            c_horizontal_line(theme, Length::Fill),
        ];

        // Page body content

        let body = match page_body {
            ChatBody::Messages => {
                let mut message_column = column![].spacing(10).padding(Padding {
                    left: 8.0,
                    right: 8.0,
                    top: 0.0,
                    bottom: 0.0,
                });

                if let Some(conversation) = conversation {
                    let ordered_conversation: Vec<_> = conversation.iter().rev().cloned().collect();

                    for message in ordered_conversation {
                        if let Some(message_element) = c_chat_message(
                            theme,
                            message,
                            &current_chat.id,
                            chat_message_options,
                            emoji_map,
                            users,
                            me,
                            user_presences,
                            show_more_options,
                            more_menu_message_id,
                            show_message_emoji_picker,
                            show_plus_emoji_picker,
                            emoji_picker_message_id,
                            search_emojis_input_value,
                            window_size,
                        ) {
                            message_column = message_column.push(message_element);
                        }
                    }
                } else {
                    if current_chat.chat_type.clone().unwrap_or("any".to_string()) == "draft" {
                        message_column = message_column.push(
                            container(
                                text("Type a message below to start your conversation.")
                                    .color(theme.colors.demo_text),
                            )
                            .width(Length::Fill)
                            .padding(Padding {
                                left: 8.0,
                                top: 6.0,
                                right: 0.0,
                                bottom: 0.0,
                            }),
                        );
                    }
                }
                container(
                    scrollable(message_column)
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new()
                                .width(theme.features.scrollbar_width)
                                .spacing(theme.features.scrollable_spacing)
                                .scroller_width(theme.features.scrollbar_width),
                        ))
                        .style(|_, _| theme.stylesheet.scrollable)
                        .id(Id::new("conversation_column"))
                        .on_scroll(Message::OnScroll),
                )
                .padding(Padding {
                    top: 8.0,
                    right: 3.0,
                    left: 0.0,
                    bottom: 0.0,
                })
                .height(Length::Fill)
            }
            ChatBody::Members => {
                let mut members_column =
                    column![]
                        .spacing(theme.features.list_spacing)
                        .padding(Padding {
                            left: 8.0,
                            right: 6.0,
                            top: 6.0,
                            bottom: 6.0,
                        });

                for member in &current_chat.members {
                    let member_id = member.mri.strip_prefix("8:orgid:").unwrap_or(&member.mri);

                    let identifier = member_id.replace(":", "");

                    let user = users.get(member_id);

                    let mut message_row = row![].width(Length::Fill).align_y(Alignment::Center);

                    let display_name = if let Some(user) = user {
                        user.display_name
                            .clone()
                            .unwrap_or("Unknown User".to_string())
                    } else {
                        "Unknown User".to_string()
                    };

                    // The Teams api *might still work when the username is wrong
                    let user_picture = c_cached_image(
                        identifier.clone(),
                        Message::FetchUserImage(
                            identifier,
                            member_id.to_string(),
                            display_name.to_string(),
                        ),
                        28.0,
                        28.0,
                        4.0,
                    );

                    let presence = user_presences.get(&member.mri);

                    message_row = message_row.push(c_picture_and_status(
                        theme,
                        user_picture,
                        presence,
                        (28.0, 28.0),
                    ));
                    message_row = message_row.push(text(display_name));

                    members_column = members_column.push(
                        container(message_row)
                            .style(|_| container::Style {
                                background: Some(theme.colors.foreground.into()),
                                border: border::rounded(4),
                                ..Default::default()
                            })
                            .padding(Padding {
                                top: 6.0,
                                right: 3.0,
                                bottom: 6.0,
                                left: 3.0,
                            }),
                    );
                }

                container(
                    scrollable(members_column)
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new()
                                .width(theme.features.scrollbar_width)
                                .spacing(theme.features.scrollable_spacing)
                                .scroller_width(theme.features.scrollbar_width),
                        ))
                        .style(|_, _| theme.stylesheet.scrollable)
                        .id(Id::new("members_column")),
                )
                .padding(padding::right(3))
                .height(Length::Fill)
            }
            ChatBody::Activity => {
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

                        if let Some(conversation) = expanded_conversations.get(&message_activity_id)
                        {
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
                                        activities_colum =
                                            activities_colum.push(mouse_area(message).on_release(
                                                Message::ToggleExpandActivity(
                                                    thread_id,
                                                    message_id,
                                                    message_activity_id,
                                                ),
                                            ));
                                    }
                                } else {
                                    activities_colum = activities_colum.push(
                                        mouse_area(text("Failed to load conversation."))
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
                                    .on_release(
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
                                .on_release(
                                    Message::ToggleExpandActivity(
                                        thread_id,
                                        message_id,
                                        message_activity_id,
                                    ),
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
                .padding(Padding {
                    top: 8.0,
                    right: 3.0,
                    left: 0.0,
                    bottom: 0.0,
                })
                .height(Length::Fill);

                activities_scrollbar.into()
            }
        };

        // Put together page content

        let mut content_page = column![];

        content_page = content_page.push(title_row_container);

        content_page = content_page.push(body);

        if let Some(chat_typing) = users_typing.get(&current_chat.id) {
            if !chat_typing.is_empty() {
                let typers = chat_typing
                    .keys()
                    .cloned()
                    .map(|item| {
                        format!(
                            "{}",
                            users
                                .get(&item.replace("8:orgid:", ""))
                                .and_then(|profile| profile.display_name.clone())
                                .unwrap_or_else(|| "Unknown User".to_string())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                content_page = content_page.push(
                    container(
                        text!("{} is typing...", typers)
                            .size(14)
                            .color(theme.colors.demo_text),
                    )
                    .padding(Padding {
                        top: 4.0,
                        right: 10.0,
                        bottom: 3.0,
                        left: 10.0,
                    })
                    .height(25),
                );
            } else {
                content_page = content_page.push(space().height(25))
            }
        } else {
            content_page = content_page.push(space().height(25))
        }

        let message_area = container(c_message_area(
            theme,
            message_area_content,
            &None,
            crate::View::Chat,
            message_area_height,
            show_message_area_emoji_picker,
            search_emojis_input_value,
            emoji_map,
            window_size,
        ))
        .padding(Padding {
            left: 8.0,
            right: 8.0,
            top: 0.0,
            bottom: 6.0,
        });

        if *page_body != ChatBody::Activity {
            content_page = content_page.push(message_area);
        }

        page = page.push(content_page);
    }

    page.into()
}
