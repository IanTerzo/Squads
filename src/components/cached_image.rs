use crate::widgets::viewport::ViewportHandler;
use crate::Message;
use bytes::Bytes;
use directories::ProjectDirs;
use iced::widget::{container, image, Space};
use iced::{Color, ContentFit, Element};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
pub fn save_cached_image(identifier: String, bytes: Bytes) {
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    cache_dir.push("image-cache");

    if !cache_dir.exists() {
        create_dir_all(&cache_dir).expect("Failed to create image-cache directory");
    }

    cache_dir.push(format!("{}.jpeg", identifier));
    if !cache_dir.exists() {
        let mut file = File::create(cache_dir).unwrap();
        let _ = file.write_all(&bytes);
    }
}

pub fn c_cached_image<'a>(
    identifier: String,
    on_enter_unique: Message,
    image_width: f32,
    image_height: f32,
) -> Element<'a, Message> {
    let mut team_picture = container(
        ViewportHandler::new(Space::new(0, 0))
            .on_enter_unique(identifier.clone(), on_enter_unique.clone()),
    )
    .style(|_| container::Style {
        background: Some(
            Color::parse("#b8b4b4")
                .expect("Background color is invalid.")
                .into(),
        ),

        ..Default::default()
    });
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut image_path = project_dirs.unwrap().cache_dir().to_path_buf();
    image_path.push("image-cache");
    image_path.push(format!("{}.jpeg", identifier));
    if Path::new(&image_path).exists() {
        team_picture = container(
            ViewportHandler::new(
                image(image_path)
                    .content_fit(ContentFit::Fill)
                    .width(image_width)
                    .height(image_height),
            )
            .on_enter_unique(identifier, on_enter_unique),
        )
    }

    team_picture.into()
}
