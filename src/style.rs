use std::str::FromStr;

use iced::{
    Color, Shadow, border,
    widget::{
        container,
        scrollable::{self, AutoScroll},
        text_input,
    },
};

#[derive(Debug)]
pub struct Stylesheet {
    pub input: text_input::Style,
    pub scrollable: scrollable::Style,
}

#[derive(Debug)]
pub struct Colors {
    pub not_set: Color, // To be used as a palceholder for colors that are not yet choosen
    pub background: Color,
    pub foreground: Color,
    pub foreground_button: Color,
    pub foreground_button_nobg_hovered: Color,
    pub foreground_button_nobg_selected: Color,
    pub foreground_surface: Color,
    pub foreground_alt: Color,
    pub line: Color,
    pub tooltip: Color,
    pub status_available: Color,
    pub status_offline: Color,
    pub status_busy: Color,
    pub status_away: Color,
    pub message_hovered: Color,
    pub text: Color,
    pub text_link: Color,
    pub demo_text: Color,
    pub text_selection: Color,
    pub accent: Color,
    pub notification: Color,
    pub emotion_selected: Color,
    pub emotion_selected_line: Color,
}

#[derive(Debug)]
pub struct Theme {
    pub colors: Colors,
    pub stylesheet: Stylesheet,
}

pub fn squads_dark() -> Theme {
    let colors = Colors {
        not_set: Color::from_rgb(1.0, 0.0, 0.0), // To be used as a palceholder for colors that are not yet choosen
        background: Color::from_str("#211F1F").expect("Color is invalid."),
        foreground: Color::from_str("#1B1A1A").expect("Color is invalid."),
        foreground_button: Color::from_str("#232222").expect("Color is invalid."),
        foreground_button_nobg_hovered: Color::from_str("#201F1F").expect("Color is invalid."),
        foreground_button_nobg_selected: Color::from_str("#323030").expect("Color is invalid."),
        foreground_surface: Color::from_str("#332E2E").expect("Color is invalid."),
        foreground_alt: Color::from_str("#272525").expect("Color is invalid."),
        line: Color::from_str("#393939").expect("Color is invalid."),
        tooltip: Color::from_str("#29292A").expect("Color is invalid."),
        status_available: Color::from_str("#4db255").expect("Color is invalid."),
        status_offline: Color::from_str("#696c65").expect("Color is invalid."),
        status_busy: Color::from_str("#a92622").expect("Color is invalid."),
        status_away: Color::from_str("#ed9612").expect("Color is invalid."),
        message_hovered: Color::from_str("#2A2929").expect("Color is invalid."),
        text: Color::WHITE,
        text_link: Color::from_str("#767df5").expect("Color is invalid."),
        demo_text: Color::from_str("#c1c1c1").expect("Color is invalid."),
        text_selection: Color::from_str("#0824bb").expect("Color is invalid."),
        accent: Color::WHITE,
        notification: Color::WHITE,
        emotion_selected: Color::from_str("#24323D").expect("Color is invalid."),
        emotion_selected_line: Color::from_str("#5057D7").expect("Color is invalid."),
    };

    let stylesheet = Stylesheet {
        input: text_input::Style {
            background: colors.foreground_surface.into(),
            border: border::rounded(6),
            icon: colors.not_set,
            placeholder: colors.demo_text,
            value: colors.text,
            selection: colors.text_selection,
        },
        scrollable: scrollable::Style {
            container: container::Style {
                background: None,
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(colors.foreground.into()),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    background: colors.accent.into(),
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(colors.foreground.into()),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    background: colors.not_set.into(),
                    border: border::rounded(10),
                },
            },
            gap: Some(colors.not_set.into()),
            auto_scroll: AutoScroll {
                background: colors.foreground.into(),
                icon: colors.not_set,
                border: border::rounded(10),
                shadow: Shadow {
                    ..Default::default()
                },
            },
        },
    };

    Theme { colors, stylesheet }
}

pub fn global_theme() -> Theme {
    squads_dark()
}
