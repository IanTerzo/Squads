use crate::api::{Channel, Team, TeamConversations};
use crate::components::{conversation::c_conversation, styled_scrollbar::c_styled_scrollbar};
use crate::utils::truncate_name;
use crate::Message;
use directories::ProjectDirs;
use iced::widget::text_editor::Content;
use iced::widget::{column, container, image, row, text, Column, MouseArea, Space};
use iced::widget::{rich_text, span, svg, text_editor};
use iced::{border, font, padding, Alignment, Color, ContentFit, Element, Font, Length, Padding};
use std::collections::HashMap;

pub fn team<'a>(
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
            let conversaton_element = c_conversation(conversation, show_replies, emoji_map);

            if let Some(conversation_element_un) = conversaton_element {
                conversation_column = conversation_column.push(conversation_element_un)
            }
        }
    }

    let conversation_scrollbar =
        container(c_styled_scrollbar(conversation_column)).height(Length::Fill);

    let message_box = container(
        container(column![
            container(
                row![
                    row![
                        container(text("Write"))
                            .style(|_| container::Style {
                                background: Some(
                                    Color::parse("#333")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                ..Default::default()
                            })
                            .padding(3)
                            .align_y(Alignment::Center),
                        container(text("Preview"))
                            .style(|_| container::Style {
                                background: Some(
                                    Color::parse("#484848")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(4),
                                ..Default::default()
                            })
                            .padding(3)
                            .align_y(Alignment::Center)
                    ]
                    .spacing(8),
                    container(
                        row![
                            row![
                                rich_text![span("B").font(Font {
                                    weight: font::Weight::Bold,
                                    ..Default::default()
                                })]
                                .size(20),
                                rich_text![span("I").font(Font {
                                    style: font::Style::Italic,
                                    ..Default::default()
                                })]
                                .size(20),
                                rich_text![span("U").underline(true)].size(20),
                                rich_text![span("S").strikethrough(true)].size(20)
                            ]
                            .spacing(8),
                            row![
                                svg("images/list.svg").width(23).height(23),
                                svg("images/list-ordered.svg").width(23).height(23)
                            ]
                            .padding(padding::top(3))
                            .spacing(8),
                            row![
                                svg("images/code.svg").width(23).height(23),
                                svg("images/text-quote.svg").width(23).height(23),
                                svg("images/link.svg").width(19).height(19),
                                svg("images/image.svg").width(19).height(19),
                                svg("images/at-sign.svg").width(19).height(19)
                            ]
                            .padding(padding::top(3))
                            .spacing(8),
                        ]
                        .spacing(20)
                    )
                    .align_right(Length::Fill)
                ]
                .padding(Padding {
                    top: 8.0,
                    right: 10.0,
                    bottom: 4.0,
                    left: 10.0
                })
            )
            .style(|_| container::Style {
                background: Some(
                    Color::parse("#484848")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(4),

                ..Default::default()
            }),
            text_editor(message_area_content)
                .padding(8)
                .height(message_area_height)
                .on_action(Message::Edit)
                .placeholder("Type your message...")
                .style(|_, _| text_editor::Style {
                    background: Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                    border: border::rounded(4),
                    icon: Color::parse("#444").expect("Icon color is invalid."),
                    placeholder: Color::parse("#666").expect("Placeholder color is invalid."),
                    value: Color::parse("#fff").expect("Value color is invalid."),
                    selection: Color::parse("#444").expect("Selection color is invalid."),
                }),
            row![
                row![
                    svg("images/smile.svg").width(20).height(20),
                    svg("images/upload.svg").width(20).height(20),
                ]
                .spacing(8),
                container(
                    MouseArea::new(
                        container(text("Send"))
                            .style(|_| container::Style {
                                background: Some(
                                    Color::parse("#525252")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(4),
                                ..Default::default()
                            })
                            .padding(4)
                            .align_y(Alignment::Center)
                    )
                    .on_release(Message::PostMessage)
                )
                .align_right(Length::Fill)
            ]
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 8.0,
                left: 10.0
            })
        ])
        .style(|_| container::Style {
            background: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
            border: border::rounded(8),
            ..Default::default()
        }),
    )
    .padding(padding::top(10));
    let content_page = column![conversation_scrollbar, message_box];

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
    row![team_info_column, content_page].spacing(10).into()
}
