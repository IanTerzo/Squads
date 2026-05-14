use std::collections::HashMap;

use iced::alignment::Vertical;
use iced::widget::{column, container, mouse_area, row, svg, text};
use iced::{Border, Element, Font, Padding, border, font};

use crate::Message;
use crate::api::Profile;
use crate::components::cached_image::c_cached_image;
use crate::components::picture_and_status::c_picture_and_status;
use crate::websockets::Presence;
use crate::{style, utils};

pub fn c_profile<'a>(
    theme: &'a style::Theme,
    me: &Profile,
    user_presences: &'a HashMap<String, Presence>,
) -> Element<'a, Message> {
    let identifier = me.id.clone().replace(":", "");

    let user_picture_profile = c_cached_image(
        identifier.clone(),
        Message::FetchUserImage(
            identifier,
            me.id.clone(),
            me.display_name.as_ref().unwrap_or(&"".to_string()).clone(),
        ),
        31.0,
        31.0,
        4.0,
    );

    let presence = user_presences.get(&me.id);

    mouse_area(
        container(
            column![
                row![
                    c_picture_and_status(theme, user_picture_profile, presence, (31.0, 31.0)),
                    column![
                        text(me.display_name.clone().unwrap_or("".to_string())).font(Font {
                            weight: font::Weight::Bold,
                            ..Default::default()
                        }),
                        text(me.job_title.clone().unwrap_or("".to_string()))
                            .size(14)
                            .color(theme.colors.demo_text)
                    ]
                ]
                .align_y(Vertical::Center)
                .spacing(6),
                container(
                    container(
                        row![
                            text("Offline"),
                            svg(utils::get_image_dir().join("chevron-right.svg"))
                                .width(19)
                                .height(19)
                        ]
                        .align_y(Vertical::Center)
                        .width(140)
                    )
                    .padding(5)
                    .style(move |_| {
                        container::Style {
                            background: Some(theme.colors.background_button.into()),
                            border: border::rounded(4.0),
                            ..Default::default()
                        }
                    })
                )
                .padding(4)
            ]
            .spacing(6),
        )
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 4.0,
            right: 6.0,
        })
        .style(move |_| container::Style {
            background: Some(theme.colors.background.into()),
            border: Border {
                color: theme.colors.line,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }),
    )
    .interaction(iced::mouse::Interaction::Idle)
    .into()
}
