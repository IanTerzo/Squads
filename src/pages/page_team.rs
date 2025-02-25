use crate::api::{Channel, Team, TeamConversations};
use crate::components::{conversation::c_conversation, message_area::c_message_area};
use crate::style::Stylesheet;
use crate::utils::truncate_name;
use crate::Message;
use directories::ProjectDirs;
use iced::widget::text_editor::Content;
use iced::widget::{column, container, image, row, scrollable, text, Column, MouseArea, Space};
use iced::{font, ContentFit, Element, Length, Padding};
use std::collections::HashMap;

pub fn team<'a>(
    theme: &'a Stylesheet,
    team: Team,
    page_channel: Channel,
    conversations: Option<TeamConversations>,
    reply_options: HashMap<String, bool>,
    emoji_map: &HashMap<String, String>,
    message_area_content: &'a Content,
    message_area_height: f32,
) -> Element<'a, Message> {
    let mut conversation_column = column![].spacing(10);

    if let Some(conversations) = conversations {
        let ordered_conversations: Vec<_> =
            conversations.reply_chains.iter().rev().cloned().collect();

        for conversation in ordered_conversations {
            let mut show_replies = false;
            if let Some(option) = reply_options.get(&conversation.id) {
                show_replies = option.clone();
            }
            let conversaton_element = c_conversation(theme, conversation, show_replies, emoji_map);

            if let Some(conversation_element_un) = conversaton_element {
                conversation_column = conversation_column.push(conversation_element_un)
            }
        }
    }

    let conversation_scrollbar = container(
        scrollable(conversation_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(8)
                    .spacing(10)
                    .scroller_width(8),
            ))
            .style(|_, _| theme.scrollable),
    )
    .height(Length::Fill);

    let message_area = c_message_area(theme, message_area_content, message_area_height);
    let content_page = column![conversation_scrollbar, message_area];

    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut image_path = project_dirs.unwrap().cache_dir().to_path_buf();
    image_path.push("image-cache");
    image_path.push(format!("{}.jpeg", team.picture_e_tag));

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

    let mut channels_coloumn: Column<Message> = column![];

    let channel_count = team.channels.len();

    for channel in team.channels.clone() {
        let page_channel_cloned = page_channel.clone();
        let channel_cloned = channel.clone();
        channels_coloumn = channels_coloumn.push(
            MouseArea::new(
                container(text(truncate_name(channel.display_name, 16)))
                    .style(move |_| {
                        if channel_cloned.id == page_channel_cloned.id {
                            theme.list_tab_selected
                        } else {
                            theme.list_tab
                        }
                    })
                    .padding(Padding::from([0, 8]))
                    .center_y(47)
                    .width(if channel_count <= 13 { 220 } else { 185 }),
            )
            .on_press(Message::OpenTeam(team.clone().id, channel.id)),
        );
        channels_coloumn = channels_coloumn.push(Space::new(10, 8.5));
    }

    let team_scrollbar = scrollable(channels_coloumn)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(8)
                .spacing(10)
                .scroller_width(8),
        ))
        .style(|_, _| theme.scrollable);

    let team_info_column = column![name_row, sidetabs, team_scrollbar].spacing(18);
    row![team_info_column, content_page].spacing(10).into()
}
