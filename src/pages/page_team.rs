use crate::api::{Channel, Profile, Team, TeamConversations};
use crate::components::{conversation::c_conversation, message_area::c_message_area};
use crate::style;
use crate::types::Emoji;
use crate::utils::truncate_name;
use crate::websockets::Presence;
use crate::Message;
use directories::ProjectDirs;
use iced::widget::scrollable::Id;
use iced::widget::text_editor::Content;
use iced::widget::{column, container, image, row, scrollable, text, Column, MouseArea, Space};
use iced::{font, padding, ContentFit, Element, Length, Padding};
use indexmap::IndexMap;
use std::collections::HashMap;

pub fn team<'a>(
    theme: &'a style::Theme,
    team: &mut Team,
    page_channel: &Channel,
    conversations: &Option<&TeamConversations>,
    reply_options: &HashMap<String, bool>,
    emoji_map: &IndexMap<String, Emoji>,
    users: &HashMap<String, Profile>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
    subject_input_content: &String,
    message_area_content: &'a Content,
    message_area_height: &f32,
) -> Element<'a, Message> {
    let mut conversation_column = column![].spacing(12).padding(Padding {
        left: 8.0,
        right: 8.0,
        top: 0.0,
        bottom: 0.0,
    });

    if let Some(conversations) = conversations {
        let ordered_conversations: Vec<_> =
            conversations.reply_chains.iter().rev().cloned().collect();

        for conversation in ordered_conversations {
            let mut show_replies = false;
            if let Some(option) = reply_options.get(&conversation.id) {
                show_replies = option.clone();
            }

            let conversaton_element = c_conversation(
                theme,
                conversation.messages.iter().rev().cloned().collect(),
                page_channel.id.clone(),
                conversation.id,
                show_replies,
                emoji_map,
                users,
                me,
                user_presences,
            );

            // let ordered_conversation: Vec<_> = c;

            if let Some(conversation_element_un) = conversaton_element {
                conversation_column = conversation_column.push(conversation_element_un)
            }
        }
    }

    let conversation_scrollbar = container(
        scrollable(conversation_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(theme.features.scrollbar_width)
                    .spacing(theme.features.scrollable_spacing)
                    .scroller_width(theme.features.scrollbar_width),
            ))
            .style(|_, _| theme.stylesheet.scrollable)
            .on_scroll(Message::OnScroll)
            .id(Id::new("conversation_column")),
    )
    .padding(padding::right(3))
    .height(Length::Fill);

    let message_area = container(c_message_area(
        theme,
        message_area_content,
        Some(subject_input_content),
        message_area_height,
    ))
    .padding(Padding {
        left: 8.0,
        right: 8.0,
        top: 0.0,
        bottom: 6.0,
    });

    let content_page = column![conversation_scrollbar, Space::new(0, 7), message_area].spacing(7);

    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut image_path = project_dirs.unwrap().cache_dir().to_path_buf();
    image_path.push("image-cache");
    image_path.push(format!(
        "{}.jpeg",
        team.picture_e_tag
            .clone()
            .unwrap_or(team.display_name.clone())
    ));

    let team_picture = image(image_path)
        .content_fit(ContentFit::Cover)
        .width(45)
        .height(45);

    let name_row = row![
        team_picture,
        column![
            text!("{}", truncate_name(team.display_name.clone(), 16)).font(font::Font {
                weight: font::Weight::Bold,
                ..Default::default()
            }),
            text!("{}", truncate_name(page_channel.display_name.clone(), 16))
        ]
        .spacing(5)
    ]
    .spacing(10);

    let sidetabs = column![text!("Class Notebook"), text!("Assignments")].spacing(8);

    let team_info = container(
        column![
            name_row,
            sidetabs,
            container(Space::new(210, 1)).style(|_| container::Style {
                background: Some(theme.colors.primary3.into()),
                ..Default::default()
            }),
        ]
        .spacing(18),
    )
    .padding(Padding {
        top: 11.0,
        left: 10.0,
        bottom: 14.0,
        right: 0.0,
    })
    .style(|_| container::Style {
        background: Some(theme.colors.primary1.into()),
        ..Default::default()
    });

    let mut channels_coloumn: Column<Message> = column![]
        .spacing(theme.features.list_spacing)
        .padding(Padding {
            right: 4.0,
            left: 6.0,
            top: 6.0,
            bottom: 6.0,
        });

    let channels_sorted = team.channels.sort_by_key(|item| item.id != team.id);
    for channel in team.channels.clone() {
        let page_channel_cloned = page_channel.clone();
        let channel_cloned = channel.clone();
        channels_coloumn = channels_coloumn.push(
            MouseArea::new(
                container(text(truncate_name(channel.display_name, 16)))
                    .style(move |_| {
                        if channel_cloned.id == page_channel_cloned.id {
                            theme.stylesheet.list_tab_selected
                        } else {
                            theme.stylesheet.list_tab
                        }
                    })
                    .padding(Padding::from([0, 8]))
                    .center_y(45)
                    .width(216),
            )
            .on_enter(Message::PrefetchTeam(team.id.clone(), channel.id.clone()))
            .on_release(Message::OpenTeam(team.id.clone(), channel.id)),
        );
    }

    let team_scrollbar = scrollable(channels_coloumn)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(theme.features.scrollbar_width)
                .spacing(theme.features.scrollable_spacing)
                .scroller_width(theme.features.scrollbar_width),
        ))
        .style(|_, _| theme.stylesheet.side_scrollable);

    let team_info_column = container(column![team_info, team_scrollbar])
        .style(|_| container::Style {
            background: Some(theme.colors.primary1.into()),
            ..Default::default()
        })
        .height(Length::Fill);

    row![team_info_column, content_page]
        .spacing(theme.features.page_row_spacing)
        .into()
}
