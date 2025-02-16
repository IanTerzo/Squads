use crate::api::Team;
use crate::Message;

use iced::widget::{column, container, row, text, text_input, Column, MouseArea, Space};
use iced::{border, padding, Alignment, Color, Element};

use crate::components::{cached_image::c_cached_image, styled_scrollbar::c_styled_scrollbar};
use crate::utils::truncate_name;

pub fn home<'a>(teams: Vec<Team>, search_teams_input_value: String) -> Element<'a, Message> {
    let mut teams_column: Column<Message> = column![].spacing(8.5);

    for team in teams {
        let team_picture = c_cached_image(
            team.picture_e_tag.clone(),
            Message::FetchTeamImage(
                team.picture_e_tag,
                team.team_site_information.group_id.clone(),
                team.display_name.clone(),
            ),
            28.0,
            28.0,
        );

        teams_column = teams_column.push(
            MouseArea::new(
                container(
                    row![
                        container(team_picture).padding(padding::left(10)),
                        text(truncate_name(team.display_name, 16)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
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
    }

    let team_scrollbar = container(c_styled_scrollbar(teams_column));
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
