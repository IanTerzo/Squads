use iced::{
    widget::{
        container, stack,
        svg::{self, Handle, Svg},
    },
    Element, Padding,
};

use crate::{style::Theme, websockets::Presence, widgets::circle::circle, Message};

fn svg_availible<'a>(theme: &Theme) -> Element<'a, Message> {
    let [r, g, b, a] = theme.colors.status_available.into_rgba8();
    let [r1, g1, b1, a1] = theme.colors.primary1.into_rgba8();

    let svg_content = format!(
        r##"
        <svg height="100" width="100" xmlns="http://www.w3.org/2000/svg">
        	<circle r="14" cx="76" cy="76" fill="rgba({},{},{},{})" />

        	<circle r="14" cx="76" cy="76" fill="none" stroke="rgba({},{},{},{})" stroke-width="8" />
        </svg>
    	"##,
        r, g, b, a, r1, g1, b1, a1
    );

    Svg::new(Handle::from_memory(svg_content.into_bytes())).into()
}

fn svg_away<'a>(theme: &Theme) -> Element<'a, Message> {
    let [r, g, b, a] = theme.colors.status_away.into_rgba8();
    let [r1, g1, b1, a1] = theme.colors.primary1.into_rgba8();

    let svg_content = format!(
        r##"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
          <circle r="14" cx="76" cy="76" fill="rgba({},{},{},{})" />

          <clipPath id="clipBottomCircle">
          	<circle r="14" cx="76" cy="76" />
          </clipPath>
          <circle r="10" cx="68" cy="68" fill="rgba({},{},{},{})" clip-path="url(#clipBottomCircle)" />

          <circle r="14" cx="76" cy="76" fill="none" stroke="rgba({},{},{},{})" stroke-width="8" />
        </svg>
    	"##,
        r, g, b, a, r1, g1, b1, a1, r1, g1, b1, a1
    );

    Svg::new(Handle::from_memory(svg_content.into_bytes())).into()
}

fn svg_busy<'a>(theme: &Theme) -> Element<'a, Message> {
    let [r, g, b, a] = theme.colors.status_busy.into_rgba8();
    let [r1, g1, b1, a1] = theme.colors.primary1.into_rgba8();

    let svg_content = format!(
        r##"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
          <circle r="14" cx="76" cy="76" fill="rgba({},{},{},{})" />
          <rect x="70" y="73" width="13" height="6" fill="rgba({},{},{},{})" />
          <circle r="14" cx="76" cy="76" fill="none" stroke="rgba({},{},{},{})" stroke-width="8" />
        </svg>
    	"##,
        r, g, b, a, r1, g1, b1, a1, r1, g1, b1, a1
    );

    Svg::new(Handle::from_memory(svg_content.into_bytes())).into()
}

fn svg_offline<'a>(theme: &Theme) -> Element<'a, Message> {
    let [r, g, b, a] = theme.colors.status_offline.into_rgba8();
    let [r1, g1, b1, a1] = theme.colors.primary1.into_rgba8();

    let svg_content = format!(
        r##"
        <svg height="100" width="100" xmlns="http://www.w3.org/2000/svg">
        	<circle r="14" cx="76" cy="76" fill="rgba({},{},{},{})" />
         	<circle r="14" cx="76" cy="76" fill="none" stroke="rgba({},{},{},{})" stroke-width="8" />
        </svg>
    	"##,
        r, g, b, a, r1, g1, b1, a1
    );

    Svg::new(Handle::from_memory(svg_content.into_bytes())).into()
}

pub fn c_picture_and_status<'a>(
    theme: &Theme,
    picture: Element<'a, Message>,
    presence: Option<&Presence>, // If presence is unknown
    picture_size: (f32, f32),
) -> Element<'a, Message> {
    container(stack![
        container(picture).padding(6),
        container(if let Some(presence) = presence {
            if let Some(activity) = &presence.presence.activity {
                match activity.as_str() {
                    "Available" => svg_availible(theme),
                    "Busy" => svg_busy(theme),
                    "DoNotDisturb" => svg_busy(theme),
                    "InACall" => svg_busy(theme),
                    "Presenting" => svg_busy(theme),
                    "Away" => svg_away(theme),
                    "BeRightBack" => svg_away(theme),
                    _ => svg_offline(theme),
                }
            } else {
                svg_offline(theme)
            }
        } else {
            svg_offline(theme)
        })
        .width(picture_size.0 + 12.0)
        .height(picture_size.1 + 12.0)
    ])
    .into()
}
