use crate::Message;
use crate::widgets::gif::Gif;
use directories::ProjectDirs;
use iced::Element;
use iced::widget::{container, image, mouse_area};

pub fn c_expanded_image<'a>(identifier: &String, image_type: &String) -> Element<'a, Message> {
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");
    let mut image_path = project_dirs.unwrap().cache_dir().to_path_buf();
    image_path.push("image-cache");
    image_path.push(format!("{}.{}", identifier, image_type));

    mouse_area(if image_type == "gif" {
        container(Gif::new(image_path))
    } else {
        container(image(image_path))
    })
    .on_enter(Message::EnterCenteredOverlay)
    .on_exit(Message::ExitCenteredOverlay)
    .into()
}
