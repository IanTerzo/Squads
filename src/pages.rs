use iced::widget::{
    column, container, image, row, scrollable, text, text_input, Column, Image, MouseArea, Space,
};
use iced::{border, padding, Color, Element};
use serde_json::Value;
use std::collections::HashMap;
mod navbar;
use crate::Message;

use navbar::navbar;

pub fn homepage(
    teams: Vec<HashMap<String, Value>>,
    search_teams_input_value: String,
) -> Element<'static, Message> {
    let mut teams_column: Column<Message> = column![];

 
        for team in teams {
                if let Some(Value::String(display_name)) = team.get("displayName") {
                    let mut overflow_display_name = display_name.clone();
                    if overflow_display_name.len() > 16 {
                        overflow_display_name
                            .replace_range(16 - 3..overflow_display_name.len(), "...");
                    }

        
                    teams_column = teams_column.push(
                        MouseArea::new(
                            container(
                                row![
                                      Element::new(Image::new(image::Handle::from_path(search_teams_input_value.clone())) //  get id is maybe not a string. TODO
                                        .width(50)
                                        .height(50)),
                                    text(overflow_display_name),

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
                            .padding(10)
                            .center_y(46)
                            .width(200),
                        )
                        .on_press(Message::Join.clone()),
                    );
                    teams_column = teams_column.push(Space::new(10, 8.5));
                }
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
    )
    .padding(20);

    let search_teams = container(
        text_input("Search teams.. .", &search_teams_input_value)
            .on_input(Message::ContentChanged)
            .padding(10)
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
    .width(240)
    .padding(padding::Padding::from([0, 20]));

    column![navbar(), search_teams, team_scrollbar].into()
}

pub fn login() -> Element<'static, Message> {
    column![
        navbar(),
        container(text("Sign in to your account on the browser window"))
            .padding(padding::Padding::from([0, 20])) // Kinda hotfix until i implement a global padding.
    ]
    .into()
}

pub fn team() -> Element<'static, Message> {
    column![].padding(10).into()
}
