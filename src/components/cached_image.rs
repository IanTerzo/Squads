use bytes::Bytes;
use iced::widget::{container, image, Space};
use iced::{Color, ContentFit, Element};
use std::{fs::File, io::Write, path::Path};

use crate::widgets::viewport::ViewportHandler;
use crate::Message;

pub fn save_cached_image(identifier: String, bytes: Bytes) {
    let filename = format!("image-cache/{}.jpeg", identifier);

    if !Path::new(&filename).exists() {
        let mut file = File::create(filename).unwrap();
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
    })
    .height(image_width)
    .width(image_height);

    let image_path = format!("image-cache/{}.jpeg", identifier);

    if Path::new(&image_path).exists() {
        team_picture = container(
            ViewportHandler::new(
                image(image_path)
                    .content_fit(ContentFit::Cover)
                    .width(28)
                    .height(28),
            )
            .on_enter_unique(identifier, on_enter_unique),
        )
        .height(image_width)
        .width(image_height)
    }

    team_picture.into()
}
