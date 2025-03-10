use crate::api::Team;
use crate::style;
use crate::Message;

use iced::widget::{column, container, row, scrollable, text, text_input, Column, MouseArea};
use iced::{padding, Alignment, Element};

use crate::components::cached_image::c_cached_image;
use crate::utils::truncate_name;

pub fn home(
    theme: &style::Theme,
    teams: Vec<Team>,
    search_teams_input_value: String,
) -> Element<Message> {
    let mut teams_column: Column<Message> = column![].spacing(8.5);

    for team in teams {
        let team_picture = c_cached_image(
            team.picture_e_tag
                .clone()
                .unwrap_or(team.display_name.clone()),
            Message::FetchTeamImage(
                team.picture_e_tag
                    .clone()
                    .unwrap_or(team.display_name.clone()),
                team.picture_e_tag.unwrap_or("".to_string()),
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
                .style(|_| theme.stylesheet.list_tab)
                .center_y(47)
                .width(220),
            )
            .on_press(Message::OpenTeam(
                team.id.clone().to_string(),
                team.id.to_string(),
            )),
        );
    }

    let team_scrollbar = container(
        scrollable(teams_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(8)
                    .spacing(10)
                    .scroller_width(8),
            ))
            .style(|_, _| theme.stylesheet.scrollable),
    );

    let search_teams = container(
        text_input("Search teams...", &search_teams_input_value)
            .on_input(Message::ContentChanged)
            .padding(8)
            .style(|_, _| theme.stylesheet.input),
    )
    .width(220)
    .padding(padding::bottom(18));

    column![search_teams, team_scrollbar].into()
}
