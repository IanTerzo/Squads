use iced::widget::{button, column, container, row, text, Column, MouseArea, Space, Text};
use iced::{Background, Color, Element, Subscription, Task, Theme};
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

fn homepage(user_teams_value: HashMap<String, Value>) -> Element<'static, Message> {
    let mut teams_column: Column<Message> = column![].padding(10);

    if let Some(Value::Array(teams_array)) = user_teams_value.get("teams") {
        for team in teams_array {
            if let Value::Object(team_obj) = team {
                if let Some(Value::String(display_name)) = team_obj.get("displayName") {
                    teams_column = teams_column.push(
                        MouseArea::new(
                            container(row![Text::new(display_name.clone()), "ico"].spacing(10))
                                .padding(10)
                                .style(container::rounded_box),
                        )
                        .on_press(Message::Join.clone()),
                    );
                    teams_column = teams_column.push(Space::new(10, 15));
                }
            }
        }
    }

    teams_column.into()
}

fn team() -> Element<'static, Message> {
    let teams = column![].padding(10).into();
    teams
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
                // Add your decrement logic here, or handle accordingly
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
            background: Color::new(0.09412, 0.09412, 0.09412, 1.0),
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
