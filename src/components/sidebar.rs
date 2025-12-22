use std::collections::HashMap;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{MouseArea, column, container, mouse_area, row, scrollable, svg, text, tooltip};
use iced::{Alignment, Border, Element, Length, Padding, border};

use crate::api::{Profile, Team};
use crate::components::cached_image::c_cached_image;
use crate::components::horizontal_line::c_horizontal_line;
use crate::components::picture_and_status::c_picture_and_status;
use crate::components::toooltip::c_tooltip;
use crate::components::vertical_line::c_vertical_line;
use crate::websockets::Presence;
use crate::{Message, Page};
use crate::{style, utils};

pub fn c_sidebar<'a>(
    theme: &'a style::Theme,
    teams: &'a Vec<Team>,
    page: &'a Page,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
) -> Element<'a, Message> {
    let mut teams_column = column![].spacing(14).padding(Padding {
        right: 8.0,
        left: 8.0,
        top: 6.0,
        bottom: 6.0,
    });
    for team in teams {
        let team_picture = tooltip(
            MouseArea::new(
                container(c_cached_image(
                    team.picture_e_tag
                        .clone()
                        .unwrap_or(team.display_name.clone()),
                    Message::FetchTeamImage(
                        team.picture_e_tag
                            .clone()
                            .unwrap_or(team.display_name.clone()),
                        team.picture_e_tag.clone().unwrap_or("".to_string()),
                        team.team_site_information.group_id.clone(),
                        team.display_name.clone(),
                    ),
                    36.0,
                    36.0,
                    4.0,
                ))
                .width(38)
                .height(38)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(|_| {
                    if page
                        .current_team_id
                        .as_ref()
                        .map_or(false, |id| *id == team.id)
                    {
                        container::Style {
                            border: Border {
                                color: theme.colors.accent,
                                width: 1.0,
                                radius: 4.into(),
                            },
                            ..Default::default()
                        }
                    } else {
                        container::Style {
                            ..Default::default()
                        }
                    }
                }),
            )
            .on_release(Message::OpenTeam(team.id.clone(), team.id.clone()))
            .on_enter(Message::PrefetchTeam(team.id.clone(), team.id.clone()))
            .interaction(iced::mouse::Interaction::Pointer),
            container(text(&team.display_name).wrapping(text::Wrapping::WordOrGlyph))
                .max_width(150)
                .style(|_| container::Style {
                    background: Some(theme.colors.tooltip.into()),
                    border: Border {
                        color: theme.colors.line,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding(Padding {
                    top: 8.0,
                    bottom: 10.0,
                    right: 10.0,
                    left: 8.0,
                }),
            tooltip::Position::Right,
        );

        teams_column = teams_column.push(team_picture);
    }

    let team_scrollbar = container(
        scrollable(teams_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(0)
                    .spacing(0)
                    .scroller_width(0),
            ))
            .style(|_, _| theme.stylesheet.scrollable),
    )
    .height(Length::Fill);

    let identifier = me.id.clone().replace(":", "");

    let user_picture = c_cached_image(
        identifier.clone(),
        Message::FetchUserImage(
            identifier,
            me.id.clone(),
            me.display_name.as_ref().unwrap_or(&"".to_string()).clone(),
        ),
        28.0,
        28.0,
        4.0,
    );
    let presence = user_presences.get(&me.id);
    let user_icon = c_picture_and_status(theme, user_picture, presence, (28.0, 28.0));

    row![
        container(
            column![
                container(
                    mouse_area(
                        svg(utils::get_image_dir().join("house.svg"))
                            .width(26)
                            .height(26),
                    )
                    .on_enter(Message::PrefetchCurrentChat)
                    .on_release(Message::OpenCurrentChat)
                    .interaction(iced::mouse::Interaction::Pointer)
                )
                .padding(Padding {
                    top: 11.0,
                    bottom: 7.0,
                    left: 0.0,
                    right: 0.0
                }),
                c_horizontal_line(&theme, 60.into()),
                team_scrollbar,
                c_horizontal_line(&theme, 60.into()),
                container(user_icon).padding(Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 0.0,
                    right: 0.0
                }),
            ]
            .spacing(4)
            .align_x(Alignment::Center),
        )
        .style(|_| container::Style {
            background: Some(theme.colors.foreground.into()),
            ..Default::default()
        }),
        c_vertical_line(theme, Length::Fill)
    ]
    .into()
}
