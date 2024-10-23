use iced::widget::{button, column, container, row, text, Column, MouseArea, Space, Text, scrollable};
use iced::{Background, Color, Element, Subscription, Task, Theme, border};
use serde_json::Value;
use std::collections::HashMap;
use std::{thread, time};

mod api;
use api::user_teams;

enum Page {
    Homepage,
    Team,
}

struct Counter {
    page: Page,
    user_teams: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum Message {
    Join,
    UserTeams,
    UserTeamsFetched(Result<HashMap<String, Value>, String>),
}

fn navbar() -> Element<'static, Message> {
    container(
        container(
            row![]
        )
        .style(|_| {
            container::Style {
                background: Some(Color::parse("#333").expect("Background color is invalid.").into()),
                border: border::rounded(10),
                ..Default::default()
            }
        })
        .width(4000) // Hotifx?
        .height(45) // Should be 50?

    )
    .padding(20)
    .width(4000) // Hotifx?
    .into()
}

fn homepage(user_teams_value: HashMap<String, Value>) -> Element<'static, Message> {
    let mut teams_column: Column<Message> = column![];

    if let Some(Value::Array(teams_array)) = user_teams_value.get("teams") {
        for team in teams_array {
            if let Value::Object(team_obj) = team {
                if let Some(Value::String(display_name)) = team_obj.get("displayName") {

                    let mut overflow_display_name = display_name.clone();
                    if overflow_display_name.len() > 16 {
                        overflow_display_name.replace_range(16-3..overflow_display_name.len(),"...");
                    }

                    teams_column = teams_column.push(
                        MouseArea::new(
                            container(row!["ico", text(overflow_display_name)]
                                .spacing(10)
                            )
                            .style(|_| {
                                container::Style {
                                    background: Some(Color::parse("#333")
                                        .expect("Background color is invalid.").into()),
                                    border: border::rounded(8   ),
                                    ..Default::default()
                                }
                            })
                            .padding(10)
                            //  .height(30)
                            .center_y(46)
                            .width(200)
                        )
                        .on_press(Message::Join.clone())
                    );
                    teams_column = teams_column.push(Space::new(10, 8.5));
                }
            }
        }
    }

    let team_scrollbar = container(
        scrollable(teams_column)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
            .width(10)
            .spacing(10)
            .scroller_width(10)
        ))
        .style(|_, _| {
            scrollable::Style {
               container:  container::Style {
                    background: Some(Color::from_rgba(0.0,0.0,0.0,0.0).into()),
                    border: border::rounded(10),
                    ..Default::default()
                },
                vertical_rail: scrollable::Rail {
                    background: Some(Color::parse("#333")
                                        .expect("Background color is invalid.").into()),
                    border: border::rounded(10),
                    scroller: scrollable::Scroller {
                        color: Color::parse("#444")
                                        .expect("Background color is invalid."),
                         border: border::rounded(10),
                    }
                },
                horizontal_rail: scrollable::Rail {
                    background: Some(Color::parse("#333")
                                        .expect("Background color is invalid.").into()),
                    border: border::rounded(10),
                    scroller: scrollable::Scroller {
                        color: Color::parse("#666")
                                        .expect("Background color is invalid."),
                         border: border::rounded(10),
                    }
                },
                gap:  Some(Color::parse("#333").expect("Background color is invalid.").into())
            }
        })

    )
    .padding(20);

    column![navbar(), team_scrollbar].into()
}

fn team() -> Element<'static, Message> {
    column![].padding(10).into()
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                page: Page::Homepage,
                user_teams: HashMap::new(),
            },
            Task::perform(user_teams(), Message::UserTeamsFetched),
        )
    }

    fn view(&self) -> Element<Message> {
        println!("View called");
        match self.page {
            Page::Homepage => homepage(self.user_teams.clone()),
            Page::Team => team(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        println!("Update called");

        match message {
            Message::Join => {
                println!("Join button pressed");
                Task::none()
            }
            Message::UserTeams => Task::perform(user_teams(), Message::UserTeamsFetched),
            Message::UserTeamsFetched(response) => {
                if let Ok(user_teams) = response {
                    self.user_teams = user_teams;
                } else {
                    println!("Error occurred fetching user teams");
                }
                Task::none()
            }
        }
    }

    fn theme(&self) -> Theme {
        let custom_palette = iced::theme::palette::Palette {
            background: Color::parse("#3c3c3c").expect("Background color is invalid."),
            text: Color::new(1.0, 0.0, 0.0, 1.0),
            primary: Color::new(1.0, 0.0, 0.0, 1.0),
            success: Color::new(1.0, 0.0, 0.0, 1.0),
            danger: Color::new(1.0, 0.0, 0.0, 1.0),
        };

        Theme::custom("Squads Dark".to_string(), custom_palette)
    }
}

pub fn main() -> iced::Result {
    iced::application("Styling - Iced", Counter::update, Counter::view)
        .theme(Counter::theme)
        .run_with(Counter::new)
}
