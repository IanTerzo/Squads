use std::collections::HashMap;

use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Font};

use crate::components::cached_image::c_cached_image;
use crate::style;
use crate::utils::truncate_name;
use crate::Message;

pub fn c_preview_message<'a>(
    theme: &'a style::Theme,
    activity: crate::api::Activity,
    window_width: f32,
    emoji_map: &HashMap<String, String>,
) -> Element<'a, Message> {
    let mut message_column = column![].spacing(20);

    let mut message_info = row![].spacing(10).align_y(Alignment::Center);

    if let Some(display_name) = activity.source_user_im_display_name {
        let mut user_id = activity.source_user_id; // This is wrong for

        if activity.activity_type == "msGraph" {
            if let Some(attributed_to_actor_id) = activity.activityContext.attributed_to_actor_id {
                user_id = attributed_to_actor_id;
            }
        }

        let identifier = user_id.clone().replace(":", "");

        let user_picture = c_cached_image(
            identifier.clone(),
            Message::FetchUserImage(identifier, user_id, display_name.clone()),
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

    message_info = message_info.push(
        text(activity.activity_type.clone())
            .size(14)
            .color(theme.colors.demo_text),
    );
    message_info = message_info.push(text(date).size(14).color(theme.colors.demo_text));
    message_info = message_info.push(text(time).size(14).color(theme.colors.demo_text));

    message_column = message_column.push(message_info);

    // TODO truncate everything
    if activity.activity_type == "mention" {
        let max_len = (window_width * 0.09) as usize;
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
    } else if activity.activity_type == "reactionInChat" {
        // message.preview // tuo messagwe
        // message.activity_subtype // emoji id

        let mut reaction_unicode = "(?)";
        if let Some(reaction_value) = emoji_map.get(&activity.activity_subtype.unwrap()) {
            reaction_unicode = reaction_value;
        }

        message_column = message_column.push(column![
            text!("> {}", activity.message_preview).color(theme.colors.demo_text),
            text!("{reaction_unicode}",).font(Font::with_name("Twemoji")),
            // show thread and mesage preview
        ]);
    } else if activity.activity_type == "msGraph" {
        // TODO: check subtype
        message_column = message_column
            .push(text!("{}", activity.message_preview).color(theme.colors.demo_text));
    } else if activity.activity_type == "replyToReply" {
        message_column =
            message_column.push(text!("Replied to a message: {}", activity.message_preview));
    } else if activity.activity_type == "thirdParty" {
        message_column = message_column
            .push(text!("{}", activity.message_preview).color(theme.colors.demo_text));
    } else if activity.activity_type == "teamMembershipChange"
        && activity.activity_subtype.unwrap() == "addedToTeam"
    {
        message_column = message_column.push(
            text!("Added to {}", activity.source_thread_topic.unwrap())
                .color(theme.colors.demo_text),
        );
    }

    container(message_column)
        .style(|_| theme.stylesheet.conversation)
        .width(iced::Length::Fill)
        .padding(20)
        .into()
}
