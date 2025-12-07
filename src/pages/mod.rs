pub mod page_chat;
pub mod page_home;
pub mod page_login;
pub mod page_team;
use crate::api::Profile;
use crate::api::Team;
use crate::components::sidebar::c_sidebar;
use crate::style;
use crate::websockets::Presence;
use crate::Message;
use iced::widget::stack;
use iced::widget::{container, row};
use iced::Element;
use std::collections::HashMap;

pub fn app<'a>(
    theme: &'a style::Theme,
    teams: &Vec<Team>,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
    content: Element<'a, Message>,
    overlay: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    if let Some(overlay) = overlay {
        stack![
            row![
                c_sidebar(theme, teams, me, user_presences),
                container(content)
            ],
            overlay
        ]
        .into()
    } else {
        stack![row![
            c_sidebar(theme, teams, me, user_presences),
            container(content)
        ]]
        .into()
    }
}
