use iced::{
    border,
    widget::{container, scrollable, text_editor, text_input},
    Color,
};

#[derive(Debug)]
pub struct Stylesheet {
    pub background_color: Color,
    pub navbar: container::Style,
    pub scrollable: scrollable::Style,
    pub list_tab: container::Style,
    pub list_tab_selected: container::Style,
    pub input: text_input::Style,
    pub primary_button: container::Style,
    pub message_area: text_editor::Style,
    pub message_area_bar: container::Style,
    pub message_area_tab: container::Style,
    pub message_area_container: container::Style,
    pub conversation: container::Style,
}

#[derive(Debug)]
struct Colors {
    not_set: Color,
    background: Color,
    text: Color,
    demo_text: Color,
    text_selection: Color,
    accent: Color,
    primary1: Color,
    primary2: Color,
    primary3: Color,
    primary1_selected: Color,
}

pub fn theme_squads_dark() -> Stylesheet {
    let colors = Colors {
        not_set: Color::from_rgb(1.0, 0.0, 0.0), // To be used as a palceholder for colors that are not yet choosen
        text: Color::WHITE,
        demo_text: Color::parse("#93a0aa").expect("Color is invalid."),
        background: Color::parse("#0d0e12").expect("Color is invalid."),
        text_selection: Color::parse("#8e8b94").expect("Color is invalid."),
        accent: Color::WHITE,
        primary1: Color::parse("#1b2023").expect("Color is invalid."),
        primary1_selected: Color::parse("#45525a").expect("Color is invalid."),
        primary2: Color::parse("#5c6265").expect("Color is invalid."),
        primary3: Color::parse("#2b3338").expect("Color is invalid."),
    };

    Stylesheet {
        background_color: colors.background,
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
        list_tab: container::Style {
            background: Some(colors.primary1.into()),
            border: border::rounded(7),
            ..Default::default()
        },
        list_tab_selected: container::Style {
            background: Some(colors.primary1_selected.into()),
            border: border::rounded(7),
            ..Default::default()
        },
        message_area_bar: container::Style {
            background: Some(colors.primary3.into()),
            border: border::rounded(4),

            ..Default::default()
        },
        message_area_tab: container::Style {
            background: Some(colors.primary1.into()),
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
    }
}
