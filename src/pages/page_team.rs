use crate::api::{Channel, Team, TeamConversations};
use crate::components::{conversation::c_conversation, styled_scrollbar::c_styled_scrollbar};
use crate::utils::truncate_name;
use crate::Message;
use directories::ProjectDirs;
use iced::widget::{column, container, image, row, text, Column, MouseArea, Space};
use iced::{border, font, Color, ContentFit, Element, Padding};
use std::collections::HashMap;

pub fn team<'a>(
    team: Team,
    page_channel: Channel,
    conversations: Option<TeamConversations>,
    reply_options: HashMap<String, bool>,
    emoji_map: &HashMap<String, String>,
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
            let conversaton_element = c_conversation(conversation, show_replies, emoji_map);
            conversation_column = conversation_column.push(conversaton_element)
        }
    }

    let conversation_scrollbar = c_styled_scrollbar(conversation_column);

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
                            container::Style {
                                background: Some(
                                    Color::parse("#4c4c4c")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(8),
                                ..Default::default()
                            }
                        } else {
                            container::Style {
                                background: Some(
                                    Color::parse("#333")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(8),
                                ..Default::default()
                            }
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

    let team_scrollbar = c_styled_scrollbar(channels_coloumn);

    let team_info_column = column![name_row, sidetabs, team_scrollbar].spacing(18);
    row![team_info_column, conversation_scrollbar]
        .spacing(10)
        .into()
}
