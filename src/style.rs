use iced::{
    border,
    widget::{container, scrollable, text_editor, text_input},
    Color,
};

#[derive(Debug)]
pub struct Stylesheet {
    pub navbar: container::Style,
    pub scrollable: scrollable::Style,
    pub chat_scrollable: scrollable::Style,
    pub list_tab: container::Style,
    pub list_tab_selected: container::Style,
    pub input: text_input::Style,
    pub primary_button: container::Style,
    pub message_area: text_editor::Style,
    pub message_area_bar: container::Style,
    pub message_area_tab_active: container::Style,
    pub message_area_tab: container::Style,
    pub message_area_container: container::Style,
    pub conversation: container::Style,
}

#[derive(Debug)]
pub struct Colors {
    pub not_set: Color,
    pub background: Color,
    pub text: Color,
    pub text_link: Color,
    pub demo_text: Color,
    pub text_selection: Color,
    pub accent: Color,
    pub primary1: Color,
    pub primary2: Color,
    pub primary3: Color,
    pub primary1_selected: Color,
    pub notification: Color,
    pub status_available: Color,
    pub status_offline: Color,
    pub status_busy: Color,
    pub status_away: Color,
}

#[derive(Debug)]
pub struct Features {
    pub scrollable_spacing: u16,
    pub page_row_spacing: u16,
    pub list_spacing: u16,
    pub scrollbar_width: u16,
}

#[derive(Debug)]
pub struct Theme {
    pub colors: Colors,
    pub features: Features,
    pub stylesheet: Stylesheet,
}

pub fn squads_dark() -> Theme {
    let colors = Colors {
        not_set: Color::from_rgb(1.0, 0.0, 0.0), // To be used as a palceholder for colors that are not yet choosen
        text: Color::WHITE,
        text_link: Color::parse("#6d74f4").expect("Color is invalid."),
        demo_text: Color::parse("#c1c1c1").expect("Color is invalid."),
        background: Color::parse("#0d0e12").expect("Color is invalid."),
        text_selection: Color::parse("#8e8b94").expect("Color is invalid."),
        accent: Color::WHITE,
        primary1: Color::parse("#1c2124").expect("Color is invalid."),
        primary1_selected: Color::parse("#30393e").expect("Color is invalid."),
        primary2: Color::parse("#161b1d").expect("Color is invalid."),
        primary3: Color::parse("#2b3338").expect("Color is invalid."),
        notification: Color::WHITE,
        status_available: Color::parse("#4db255").expect("Color is invalid."),
        status_offline: Color::parse("#696c65").expect("Color is invalid."),
        status_busy: Color::parse("#a92622").expect("Color is invalid."),
        status_away: Color::parse("#ed9612").expect("Color is invalid."),
    };

    let features = Features {
        scrollable_spacing: 12,
        page_row_spacing: 13,
        list_spacing: 10,
        scrollbar_width: 7,
    };

    let stylesheet = Stylesheet {
        navbar: container::Style {
            background: Some(colors.primary1.into()),
            //border: border::rounded(10),
            ..Default::default()
        },
        input: text_input::Style {
            background: colors.primary1.into(),
            border: border::rounded(8),
            icon: colors.not_set,
            placeholder: colors.demo_text,
            value: colors.text,
            selection: colors.text_selection,
        },
        scrollable: scrollable::Style {
            container: container::Style {
                background: Some(colors.background.into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(colors.primary1.into()),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: colors.accent,
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(colors.primary1.into()),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: colors.not_set,
                    border: border::rounded(10),
                },
            },
            gap: Some(colors.not_set.into()),
        },
        chat_scrollable: scrollable::Style {
            container: container::Style {
                background: Some(colors.background.into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(colors.primary1.into()),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: colors.accent,
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(colors.primary1.into()),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: colors.not_set,
                    border: border::rounded(10),
                },
            },
            gap: Some(colors.not_set.into()),
        },
        list_tab: container::Style {
            background: Some(colors.primary1.into()),
            border: border::rounded(6),
            ..Default::default()
        },
        list_tab_selected: container::Style {
            background: Some(colors.primary1_selected.into()),
            border: border::rounded(6),
            ..Default::default()
        },
        message_area_bar: container::Style {
            background: Some(colors.primary3.into()),
            border: border::rounded(4),

            ..Default::default()
        },
        message_area_tab_active: container::Style {
            background: Some(colors.primary1_selected.into()),
            ..Default::default()
        },
        message_area_tab: container::Style {
            background: Some(colors.primary3.into()),
            ..Default::default()
        },
        message_area: text_editor::Style {
            background: colors.primary1.into(),
            border: border::rounded(4),
            icon: colors.not_set,
            placeholder: colors.demo_text,
            value: colors.text,
            selection: colors.text_selection,
        },
        message_area_container: container::Style {
            background: Some(colors.primary1.into()),
            border: border::rounded(8),
            ..Default::default()
        },
        primary_button: container::Style {
            background: Some(colors.primary2.into()),
            border: border::rounded(4),
            ..Default::default()
        },
        conversation: container::Style {
            background: Some(colors.primary1.into()),
            border: border::rounded(8),
            ..Default::default()
        },
    };

    Theme {
        colors,
        features,
        stylesheet,
    }
}

pub fn global_theme() -> Theme {
    squads_dark()
}
