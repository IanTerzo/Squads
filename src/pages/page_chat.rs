use std::collections::HashMap;

use crate::api::{self, Chat, Profile};
use crate::components::picture_and_status::c_picture_and_status;
use crate::components::{
    cached_image::c_cached_image, chat_message::c_chat_message, message_area::c_message_area,
};
use crate::parsing::parse_content_emojis;
use crate::utils::truncate_name;
use crate::websockets::Presence;
use crate::widgets::circle::circle;
use crate::Message;
use crate::{style, ChatBody};

use iced::task::Handle;
use iced::widget::scrollable::Id;
use iced::widget::text_editor::Content;
use iced::widget::{checkbox, column, container, mouse_area, row, stack, svg, text_input, Space};
use iced::widget::{scrollable, text};
use iced::Alignment::Center;
use iced::{border, padding, Alignment, Color, Element, Length, Padding};

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
            )
        } else {
            container(container(Space::new(0, 0)))
                .style(|_| container::Style {
                    background: Some(
                        Color::parse("#b8b4b4")
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
    add_users_cheked: &HashMap<String, bool>,
    chats: &'a Vec<Chat>,
    conversation: &Option<&Vec<api::Message>>,
    chat_message_options: &'a HashMap<String, bool>,
    emoji_map: &'a HashMap<String, String>,
    users: &'a HashMap<String, Profile>,
    user_presences: &'a HashMap<String, Presence>,
    me: &'a Profile,
    search_chats_input_value: String,
    search_users_input_value: String,
    message_area_content: &'a Content,
    message_area_height: &f32,
    page_body: &'a ChatBody,
) -> Element<'a, Message> {
    let mut chats_column = if *page_body == ChatBody::Start {
        column![container(
            row![
                svg("images/message-square.svg").width(18).height(18),
                text("New chat").color(theme.colors.demo_text)
            ]
            .align_y(Center)
            .spacing(10)
        )
        .style(move |_| { theme.stylesheet.list_tab_selected })
        .center_y(47)
        .padding(12)
        .width(220)]
        .spacing(theme.features.list_spacing)
        .padding(Padding {
            right: 4.0,
            left: 6.0,
            top: 6.0,
            bottom: 6.0,
        })
    } else {
        column![]
            .spacing(theme.features.list_spacing)
            .padding(Padding {
                right: 4.0,
                left: 6.0,
                top: 6.0,
                bottom: 6.0,
            })
    };

    for chat in chats {
        let chat_title = get_chat_title(&chat, &me.id, &users);
        if !chat_title
            .to_lowercase()
            .starts_with(&search_chats_input_value.to_lowercase())
        {
            continue;
        }

        let mut chat_items = row![].align_y(Alignment::Center);

        if let Some(is_read) = chat.is_read {
            if !is_read {
                chat_items = chat_items.push(
                    container(circle(2.5, theme.colors.notification)).padding(Padding {
                        top: 0.0,
                        right: 3.0,
                        bottom: 0.0,
                        left: 3.0,
                    }),
                )
            }
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

            chat_items =
                chat_items.push(c_picture_and_status(theme, picture, presence, (28.0, 28.0)));
        } else {
            chat_items = chat_items.push(container(picture).padding(6));
        }

        let mut chat_info_column = column![parse_content_emojis(truncate_name(chat_title, 20))];

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
                        if chat.id == current_chat.id && *page_body != ChatBody::Start {
                            theme.stylesheet.list_tab_selected
                        } else {
                            theme.stylesheet.list_tab
                        }
                    } else {
                        theme.stylesheet.list_tab
                    }
                })
                .center_y(47)
                .width(220),
        )
        .on_enter(Message::PrefetchChat(chat.id.clone()))
        .on_release(Message::OpenChat(chat.id.clone()));

        chats_column = chats_column.push(chat_item);
    }

    let chats_scrollable = scrollable(chats_column)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(theme.features.scrollbar_width)
                .spacing(theme.features.scrollable_spacing)
                .scroller_width(theme.features.scrollbar_width),
        ))
        .style(|_, _| theme.stylesheet.side_scrollable);

    let chat_options = column![
        container(
            row![
                container(
                    text_input("Search chats...", &search_chats_input_value)
                        .on_input(Message::SearchChatsContentChanged)
                        .padding(6)
                        .style(|_, _| theme.stylesheet.input),
                )
                .width(188),
                mouse_area(svg("images/square-pen.svg").width(22).height(22))
                    .on_release(Message::ToggleNewChatMenu)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(Padding {
            top: 8.0,
            bottom: 7.0,
            left: 7.0,
            right: 7.0
        })
        .style(|_| container::Style {
            background: Some(theme.colors.primary1.into()),
            ..Default::default()
        }),
        container(Space::new(Length::Fill, 1)).style(|_| container::Style {
            background: Some(theme.colors.primary3.into()),
            ..Default::default()
        }),
        container(Space::new(Length::Fill, 2)).style(|_| container::Style {
            background: Some(theme.colors.primary1.into()),
            ..Default::default()
        })
    ];

    let mut side_panel = container(column![chat_options, chats_scrollable].width(230))
        .style(|_| container::Style {
            background: Some(theme.colors.primary1.into()),
            ..Default::default()
        })
        .height(Length::Fill);

    let mut page = row![side_panel].spacing(theme.features.page_row_spacing);

    if let Some(current_chat) = current_chat {
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
                    chat_message_options,
                    emoji_map,
                    users,
                    me,
                    user_presences,
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

        let title = truncate_name(get_chat_title(&current_chat, &me.id, &users), 52);
        let picture = get_chat_picture(&current_chat, &me.id, &users);

        let title_row = if *page_body != ChatBody::Start {
            row![
                Space::with_width(2), // Silly...
                picture,
                Space::with_width(15),
                parse_content_emojis(title),
                if let Some(is_one_on_one) = current_chat.is_one_on_one {
                    if !is_one_on_one {
                        row![
                            Space::with_width(8),
                            svg("images/pencil.svg").width(17).height(17),
                        ]
                    } else {
                        row![]
                    }
                } else {
                    row![]
                },
                container(
                    row![
                        mouse_area(svg("images/users-round.svg").width(19).height(19))
                            .on_release(Message::ToggleShowChatMembers),
                        mouse_area(svg("images/user-round-plus.svg").width(19).height(19))
                            .on_release(Message::ToggleShowChatAdd),
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
            .align_y(Alignment::Center)
        } else {
            row![
                Space::with_width(2), // Silly...
                text("Start chat"),
            ]
            .padding(Padding {
                top: 10.0,
                right: 30.0,
                bottom: 10.0,
                left: 14.0,
            })
            .align_y(Alignment::Center)
        };

        let title_row_container = column![
            container(title_row).style(|_| container::Style {
                background: Some(theme.colors.primary2.into()),
                ..Default::default()
            }),
            container(Space::new(Length::Fill, 1)).style(|_| container::Style {
                background: Some(theme.colors.primary3.into()),
                ..Default::default()
            })
        ];

        let mut members_column = column![]
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
                        background: Some(theme.colors.primary1.into()),
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

        let body =
            match page_body {
                ChatBody::Messages => container(
                    scrollable(message_column)
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new()
                                .width(theme.features.scrollbar_width)
                                .spacing(theme.features.scrollable_spacing)
                                .scroller_width(theme.features.scrollbar_width),
                        ))
                        .style(|_, _| theme.stylesheet.chat_scrollable)
                        .id(Id::new("conversation_column"))
                        .on_scroll(Message::OnScroll),
                )
                .padding(Padding {
                    top: 8.0,
                    right: 3.0,
                    left: 0.0,
                    bottom: 0.0,
                })
                .height(Length::Fill),
                ChatBody::Add | ChatBody::Start => {
                    let mut user_column =
                        column![]
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

                        // Do not show user if already part of group or chat (except if in the start chat menu)
                        if current_chat
                            .members
                            .iter()
                            .any(|member| &member.mri.replace("8:orgid:", "") == user.0)
                            && *page_body != ChatBody::Start
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
                            container(checkbox("", is_checked).style(|_, _| checkbox::Style {
                                background: theme.colors.primary3.into(),
                                border: border::rounded(2),
                                icon_color: theme.colors.text,
                                text_color: None,
                            }))
                            .align_right(Length::Fill)
                        ]
                        .width(Length::Fill)
                        .align_y(Alignment::Center);
                        user_column = user_column
                            .push(
                                mouse_area(
                                    container(profile_row)
                                        .style(|_| container::Style {
                                            background: Some(theme.colors.primary1.into()),
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
                                .on_release(
                                    Message::ToggleUserCheckbox(is_checked, user.0.to_string()),
                                ),
                            )
                            .into();
                    }

                    container(
                        column![
                            row![
                                text_input("Search users...", &search_users_input_value)
                                    .on_input(Message::SearchUsersContentChanged)
                                    .padding(10)
                                    .style(|_, _| theme.stylesheet.input),
                                mouse_area(
                                    container(if *page_body == ChatBody::Start {
                                        text("Start")
                                    } else {
                                        text("Add")
                                    })
                                    .padding(Padding {
                                        top: 8.0,
                                        bottom: 8.0,
                                        left: 13.0,
                                        right: 13.0
                                    })
                                    .style(|_| {
                                        container::Style {
                                            background: Some(theme.colors.primary1.into()),
                                            border: border::rounded(4),
                                            ..Default::default()
                                        }
                                    })
                                )
                                .on_release(
                                    if !current_chat.is_one_on_one.unwrap_or(false)
                                        && *page_body != ChatBody::Start
                                    {
                                        Message::AddToGroupChat(
                                            current_chat.id.clone(),
                                            add_users_cheked
                                                .iter()
                                                .filter_map(|(key, &val)| {
                                                    if val {
                                                        Some(key.clone())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect(),
                                        )
                                    } else {
                                        if add_users_cheked.len() > 0 {
                                            Message::StartChat({
                                                let mut users: Vec<String> = add_users_cheked
                                                    .iter()
                                                    .filter_map(|(key, &val)| {
                                                        if val {
                                                            Some(key.clone())
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                    .collect();

                                                if *page_body != ChatBody::Start {
                                                    users.push(
                                                        current_chat
                                                            .members
                                                            .iter()
                                                            .find(|member| {
                                                                &member.mri.replace("8:orgid:", "")
                                                                    != &me.id
                                                            })
                                                            .unwrap()
                                                            .mri
                                                            .clone()
                                                            .replace("8:orgid:", ""),
                                                    );
                                                }

                                                users
                                            })
                                        } else {
                                            Message::DoNothing(())
                                        }
                                    }
                                )
                            ]
                            .align_y(Alignment::Center)
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
                                .style(|_, _| theme.stylesheet.chat_scrollable)
                                .id(Id::new("members_column"))
                        ]
                        .spacing(6),
                    )
                    .padding(padding::right(3))
                    .height(Length::Fill)
                }
                ChatBody::Members => container(
                    scrollable(members_column)
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new()
                                .width(theme.features.scrollbar_width)
                                .spacing(theme.features.scrollable_spacing)
                                .scroller_width(theme.features.scrollbar_width),
                        ))
                        .style(|_, _| theme.stylesheet.chat_scrollable)
                        .id(Id::new("members_column")),
                )
                .padding(padding::right(3))
                .height(Length::Fill),
            };

        let mut content_page = column![];

        if *page_body != ChatBody::Start {
            content_page = content_page.push(title_row_container);
        }

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
                content_page = content_page.push(Space::new(0, 25))
            }
        } else {
            content_page = content_page.push(Space::new(0, 25))
        }

        let message_area = container(c_message_area(
            theme,
            message_area_content,
            message_area_height,
        ))
        .padding(Padding {
            left: 8.0,
            right: 8.0,
            top: 0.0,
            bottom: 6.0,
        });

        if *page_body != ChatBody::Start {
            content_page = content_page.push(message_area);
        }

        page = page.push(content_page);
    }

    page.into()
}
