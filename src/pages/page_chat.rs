use std::collections::HashMap;

use crate::api::{self, Chat, Profile};
use crate::components::{
    cached_image::c_cached_image, chat_message::c_chat_message, message_area::c_message_area,
};
use crate::style;
use crate::utils::truncate_name;
use crate::websockets::Presence;
use crate::widgets::circle::circle;
use crate::Message;

use iced::task::Handle;
use iced::widget::scrollable::Id;
use iced::widget::text_editor::Content;
use iced::widget::{column, container, mouse_area, row, stack, text_input, Space};
use iced::widget::{scrollable, text};
use iced::{padding, Alignment, Color, Element, Length, Padding};

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
    chats: &'a Vec<Chat>,
    conversation: &Option<&Vec<api::Message>>,
    chat_message_options: &'a HashMap<String, bool>,
    emoji_map: &'a HashMap<String, String>,
    users: &'a HashMap<String, Profile>,
    user_presences: &'a HashMap<String, Presence>,
    me: &'a Profile,
    search_chats_input_value: String,
    message_area_content: &'a Content,
    message_area_height: &f32,
) -> Element<'a, Message> {
    let mut chats_column = column![].spacing(theme.features.list_spacing);

    let mut chat_list_empty = true;

    for chat in chats {
        let chat_title = get_chat_title(&chat, &me.id, &users);
        if !chat_title
            .to_lowercase()
            .starts_with(&search_chats_input_value.to_lowercase())
        {
            continue;
        }

        chat_list_empty = false;

        let mut chat_items = row![].align_y(Alignment::Center);

        if let Some(is_read) = chat.is_read {
            if !is_read {
                chat_items = chat_items.push(
                    container(circle(2.5, theme.colors.notification)).padding(Padding {
                        top: 0.0,
                        right: 4.0,
                        bottom: 0.0,
                        left: 4.0,
                    }),
                )
            }
        }

        let picture = if let Some(is_read) = chat.is_read {
            if !is_read {
                container(get_chat_picture(&chat, &me.id, &users)).padding(Padding {
                    top: 0.0,
                    right: 9.0,
                    bottom: 0.0,
                    left: 0.0,
                })
            } else {
                container(get_chat_picture(&chat, &me.id, &users)).padding(Padding {
                    top: 0.0,
                    right: 9.0,
                    bottom: 0.0,
                    left: 13.0,
                })
            }
        } else {
            container(get_chat_picture(&chat, &me.id, &users)).padding(Padding {
                top: 0.0,
                right: 9.0,
                bottom: 0.0,
                left: 13.0,
            })
        };

        if chat.members.len() == 2 {
            let presence = user_presences.get(
                &chat
                    .members
                    .iter()
                    .find(|member| member.mri != format!("8:orgid:{}", me.id))
                    .unwrap()
                    .mri,
            );

            chat_items = chat_items.push(stack![
                container(picture).padding(Padding {
                    top: 5.0,
                    bottom: 5.0,
                    right: 0.0,
                    left: 0.0
                }),
                container(circle(
                    5.5,
                    if let Some(presence) = presence {
                        if let Some(activity) = &presence.presence.activity {
                            match activity.as_str() {
                                "Available" => theme.colors.status_available,
                                "Busy" => theme.colors.status_busy,
                                "DoNotDisturb" => theme.colors.status_busy,
                                "InACall" => theme.colors.status_busy,
                                "Presenting" => theme.colors.status_busy,
                                "Away" => theme.colors.status_away,
                                "BeRightBack" => theme.colors.status_away,
                                _ => theme.colors.status_offline,
                            }
                        } else {
                            theme.colors.status_offline
                        }
                    } else {
                        theme.colors.status_offline
                    }
                ))
                .padding(Padding {
                    top: 24.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: if let Some(is_read) = chat.is_read {
                        if !is_read {
                            19.0
                        } else {
                            32.0
                        }
                    } else {
                        32.0
                    }
                })
            ]);
        } else {
            chat_items = chat_items.push(picture);
        }

        let mut chat_info_column = column![text(truncate_name(chat_title, 20))];
        if let Some(users_typing) = users_typing.get(&chat.id) {
            if users_typing.into_iter().len() > 0 {
                chat_info_column = chat_info_column
                    .push(text("is typing...").size(14).color(theme.colors.demo_text));
            }
        }
        chat_items = chat_items.push(chat_info_column);

        let chat_item = mouse_area(
            container(chat_items)
                .style(move |_| {
                    if let Some(current_chat) = current_chat {
                        if chat.id == current_chat.id {
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
        .style(|_, _| theme.stylesheet.scrollable);

    let search_chats = container(
        text_input("Search chats...", &search_chats_input_value)
            .on_input(Message::SearchChatsContentChanged)
            .padding(8)
            .style(|_, _| theme.stylesheet.input),
    )
    .width(220)
    .padding(padding::bottom(18));

    let mut side_panel = column![search_chats, chats_scrollable];

    // Mantain the same padding as the scrollbar
    if chat_list_empty {
        side_panel = side_panel.padding(padding::right(19));
    }

    let mut page = row![side_panel].spacing(theme.features.page_row_spacing);

    if let Some(current_chat) = current_chat {
        let mut message_column = column![].spacing(8);

        if let Some(conversation) = conversation {
            let ordered_conversation: Vec<_> = conversation.iter().rev().cloned().collect();

            for message in ordered_conversation {
                if let Some(message_element) = c_chat_message(
                    theme,
                    message,
                    chat_message_options,
                    emoji_map,
                    users,
                    user_presences,
                ) {
                    message_column = message_column.push(message_element);
                }
            }
        };

        let conversation_scrollbar = container(
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
        .height(Length::Fill);

        let title = get_chat_title(&current_chat, &me.id, &users);
        let picture = get_chat_picture(&current_chat, &me.id, &users);
        let tile_row = row![picture, text(title)].spacing(15).padding(Padding {
            top: 0.0,
            right: 14.0,
            bottom: 6.0,
            left: 14.0,
        });

        let mut content_page = column![
            container(tile_row).padding(padding::bottom(14)),
            conversation_scrollbar,
        ];

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

        let message_area = c_message_area(theme, message_area_content, message_area_height);

        content_page = content_page.push(message_area);

        page = page.push(content_page);
    }

    page.into()
}
