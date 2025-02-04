use crate::api::{Channel, Team, TeamConversations};
use crate::components::conversation::c_conversation;
use crate::utils::truncate_name;
use crate::Message;

use iced::widget::{column, container, image, row, scrollable, text, Column, MouseArea, Space};

use iced::{border, font, Color, ContentFit, Element, Padding};

pub fn team<'a>(
    team: Team,
    page_channel: Channel,
    conversations: Option<TeamConversations>,
) -> Element<'a, Message> {
    let mut conversation_column = column![].spacing(10);

    if let Some(conversations) = conversations {
        let ordered_conversations: Vec<_> =
            conversations.reply_chains.iter().rev().cloned().collect();

        for conversation in ordered_conversations {
            let conversaton_element = c_conversation(conversation, true);
            conversation_column = conversation_column.push(conversaton_element)
        }
    }

    // TODO: make it into a component
    let conversation_scrollbar = scrollable(conversation_column)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(10)
                .spacing(10)
                .scroller_width(10),
        ))
        .style(|_, _| scrollable::Style {
            container: container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0).into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#444").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#666").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            gap: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
        });

    let image_path = format!("image-cache/{}.jpeg", team.picture_e_tag);

    let team_picture = image(image_path)
        .content_fit(ContentFit::Cover)
        .width(45)
        .height(45);

    let name_row = row![
        team_picture,
        column![
            text!("{}", truncate_name(&team.display_name, 16)).font(font::Font {
                weight: font::Weight::Bold,
                ..Default::default()
            }),
            text!("{}", truncate_name(&page_channel.display_name, 16))
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
                container(text(truncate_name(&channel.display_name, 16)))
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

    let team_scrollbar = scrollable(channels_coloumn)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(10)
                .spacing(10)
                .scroller_width(10),
        ))
        .style(|_, _| scrollable::Style {
            container: container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0).into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#444").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#666").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            gap: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
        });

    let team_info_column = column![name_row, sidetabs, team_scrollbar].spacing(18);
    row![team_info_column, conversation_scrollbar]
        .spacing(10)
        .into()
}
