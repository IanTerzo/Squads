use crate::Message;
use crate::widgets::{gif::Gif, viewport::ViewportHandler};
use bytes::Bytes;
use directories::ProjectDirs;
use iced::widget::{container, image, space};
use iced::{Color, ContentFit, Element};
use std::str::FromStr;
use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
};
pub fn save_cached_image(identifier: String, extension: &str, bytes: Bytes) {
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut cache_dir = project_dirs.unwrap().cache_dir().to_path_buf();
    cache_dir.push("image-cache");

    if !cache_dir.exists() {
        create_dir_all(&cache_dir).expect("Failed to create image-cache directory");
    }

    cache_dir.push(format!("{}.{}", identifier, extension));
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
    border_radius: f32,
) -> Element<'a, Message> {
    let mut team_picture = container(
        ViewportHandler::new(space()).on_enter_unique(identifier.clone(), on_enter_unique.clone()),
    )
    .style(|_| container::Style {
        background: Some(
            Color::from_str("#b8b4b4")
                .expect("Background color is invalid.")
                .into(),
        ),

        ..Default::default()
    })
    .width(image_width)
    .height(image_height);

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
                    .height(image_height)
                    .border_radius(border_radius),
            )
            .on_enter_unique(identifier, on_enter_unique),
        )
    }

    team_picture.into()
}

pub fn c_cached_gif<'a>(
    identifier: String,
    on_enter_unique: Message,
    image_width: f32,
    image_height: f32,
) -> Element<'a, Message> {
    let mut team_picture = container(
        ViewportHandler::new(space()).on_enter_unique(identifier.clone(), on_enter_unique.clone()),
    )
    .style(|_| container::Style {
        background: Some(
            Color::from_str("#b8b4b4")
                .expect("Background color is invalid.")
                .into(),
        ),

        ..Default::default()
    })
    .width(image_width)
    .height(image_height);
    let project_dirs = ProjectDirs::from("", "ianterzo", "squads");

    let mut image_path = project_dirs.unwrap().cache_dir().to_path_buf();
    image_path.push("image-cache");
    image_path.push(format!("{}.gif", identifier));
    if Path::new(&image_path).exists() {
        team_picture = container(
            ViewportHandler::new(
                Gif::new(image_path)
                    .content_fit(ContentFit::Fill)
                    .width(image_width.into())
                    .height(image_height.into()),
            )
            .on_enter_unique(identifier, on_enter_unique),
        )
    }

    team_picture.into()
}
