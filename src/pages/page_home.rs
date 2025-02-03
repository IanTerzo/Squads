use crate::api::Team;
use crate::Message;

use iced::widget::{
    column, container, image, row, scrollable, text,
    text_input, Column, MouseArea, Space,
};
use iced::{border, padding, Color, ContentFit, Element};

use crate::utils::truncate_name;
use crate::widgets::viewport::ViewportHandler;
use std::path::Path;

pub fn home<'a>(teams: Vec<Team>, search_teams_input_value: String) -> Element<'a, Message> {
    let mut teams_column: Column<Message> = column![];

    for team in teams {
        let mut team_picture = container(ViewportHandler::new(Space::new(0, 0)).on_enter_unique(
            team.id.clone(),
            Message::FetchTeamImage(
                team.picture_e_tag.clone(),
                team.team_site_information.group_id.clone(),
                team.display_name.clone(),
            ),
        ))
        .style(|_| container::Style {
            background: Some(
                Color::parse("#b8b4b4")
                    .expect("Background color is invalid.")
                    .into(),
            ),

            ..Default::default()
        })
        .height(28)
        .width(28);

        let image_path = format!("image-cache/{}.jpeg", team.picture_e_tag);

        if Path::new(&image_path).exists() {
            team_picture = container(
                ViewportHandler::new(
                    image(image_path)
                        .content_fit(ContentFit::Cover)
                        .width(28)
                        .height(28),
                )
                .on_enter_unique(
                    team.id.clone(),
                    Message::FetchTeamImage(
                        team.picture_e_tag,
                        team.team_site_information.group_id,
                        team.display_name.clone(),
                    ),
                ),
            )
            .height(28)
            .width(28)
        }

        teams_column = teams_column.push(
            MouseArea::new(
                container(
                    row![
                        container(team_picture).padding(padding::left(10)),
                        text(truncate_name(&team.display_name, 16)),
                    ]
                    .spacing(10),
                )
                .style(|_| container::Style {
                    background: Some(
                        Color::parse("#333")
                            .expect("Background color is invalid.")
                            .into(),
                    ),
                    border: border::rounded(8),
                    ..Default::default()
                })
                .center_y(47)
                .width(220),
            )
            .on_press(Message::OpenTeam(
                team.id.clone().to_string(),
                team.id.to_string(),
            )),
        );
        teams_column = teams_column.push(Space::new(10, 8.5));
    }

    let team_scrollbar = container(
        scrollable(teams_column)
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
            }),
    );
    //.padding(20);

    let search_teams = container(
        text_input("Search teams.. .", &search_teams_input_value)
            .on_input(Message::ContentChanged)
            .padding(8)
            .style(|_, _| text_input::Style {
                background: Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
                border: border::rounded(8),
                icon: Color::parse("#444").expect("Background color is invalid."),
                placeholder: Color::parse("#666").expect("Background color is invalid."),
                value: Color::parse("#fff").expect("Background color is invalid."),
                selection: Color::parse("#444").expect("Background color is invalid."),
            }),
    )
    .width(220)
    .padding(padding::bottom(18));

    column![search_teams, team_scrollbar].into()
}
