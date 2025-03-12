use iced::widget::{column, container, row, text};
use iced::{Alignment, Element};

use crate::components::cached_image::c_cached_image;
use crate::style;
use crate::utils::truncate_name;
use crate::Message;
use scraper::{Html, Selector};

pub fn c_preview_message<'a>(
    theme: &'a style::Theme,
    activity: crate::api::Activity,
    window_width: f32,
) -> Element<'a, Message> {
    let mut message_column = column![].spacing(20);

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    if let Some(display_name) = activity.source_user_im_display_name {
        let user_id = activity.source_user_id;
        let user_picture = c_cached_image(
            user_id.clone(),
            Message::FetchUserImage(user_id.clone(), display_name.clone()),
            31.0,
            31.0,
        );

        message_info = message_info.push(user_picture);
        message_info = message_info.push(text!("{}", display_name));
    }

    let parsed_time: Vec<&str> = activity.activity_timestamp.split("T").collect();
    let date = parsed_time[0].replace("-", "/");
    let time_chunks: Vec<&str> = parsed_time[1].split(":").collect();
    let time = format!("{}:{}", time_chunks[0], time_chunks[1]);

    message_info = message_info.push(text(date).size(14).color(theme.colors.demo_text));
    message_info = message_info.push(text(time).size(14).color(theme.colors.demo_text));

    message_column = message_column.push(message_info);

    if activity.activity_type == "mention" {
        let max_len = (window_width * 0.09).round() as usize;
        println!("{max_len}");
        let mut lines = activity.message_preview.split("\n");
        let mut first_line = lines.nth(0).unwrap().to_string();

        if lines.count() >= 1 && first_line.len() < max_len {
            first_line = format!(
                "{}...",
                first_line.strip_suffix('\r').unwrap_or(first_line.as_str())
            );
        } else {
            first_line = truncate_name(first_line, max_len);
        }

        message_column = message_column.push(text(first_line).color(theme.colors.demo_text));
    } else if activity.activity_type == "teamMembershipChange"
        && activity.activity_subtype.unwrap() == "addedToTeam"
    {
        message_column = message_column.push(text!(
            "Added to team: {}",
            activity.source_thread_topic.unwrap()
        ));
    }

    container(message_column)
        .style(|_| theme.stylesheet.conversation)
        .width(iced::Length::Fill)
        .padding(20)
        .into()
}
