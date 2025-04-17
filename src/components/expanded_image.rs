use crate::widgets::gif::{self, Gif};
use crate::Message;
use directories::ProjectDirs;
use iced::widget::{container, image, mouse_area, text};
use iced::{Color, Element, Length};

pub fn c_expanded_image<'a>(identifier: String, image_type: String) -> Element<'a, Message> {
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");
    let mut image_path = project_dirs.unwrap().cache_dir().to_path_buf();
    image_path.push("image-cache");
    image_path.push(format!("{}.{}", identifier, image_type));

    mouse_area(
        container(
            if image_type == "gif" {
                container(Gif::new(image_path))
            } else {
                container(image(image_path))
            }
            .padding(80),
        )
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.97).into()),

            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill),
    )
    .on_release(Message::StopExpandImage)
    .into()
}
