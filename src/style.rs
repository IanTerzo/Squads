use iced::{
    border,
    widget::{container, scrollable, text, text_editor, text_input},
    Color,
};

#[derive(Debug)]
pub struct Stylesheet {
    pub background_color: Color,
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
struct Palette {
    gray1: Color,
    gray2: Color,
    gray3: Color,
    gray4: Color,
    gray5: Color,
    white1: Color,
    blue2: Color,
}

#[derive(Debug)]
struct Colors {
    not_set: Color,
    background: Color,
    text: Color,
    text_selection: Color,
    primary1: Color,
    primary2: Color,
    primary3: Color,
    primary1_selected: Color,
}

pub fn theme_squads_dark() -> Stylesheet {
    let palette = Palette {
        gray1: Color::parse("#444").expect("Color is invalid."),
        gray2: Color::parse("#333").expect("Color is invalid."),
        gray3: Color::parse("#222").expect("Color is invalid."),
        gray4: Color::parse("#4c4c4c").expect("Color is invalid."),
        gray5: Color::parse("#4c4c4c").expect("Color is invalid."),

        white1: Color::parse("#fff").expect("Color is invalid."),
        blue2: Color::parse("#8c8c8c").expect("Color is invalid."),
    };

    let colors = Colors {
        not_set: Color::from_rgb(1.0, 0.0, 0.0), // To be used as a palceholder for colors that are not yet choosen
        text: palette.white1,
        background: palette.gray1,
        text_selection: palette.blue2,
        primary1: palette.gray2,
        primary2: palette.gray3,
        primary3: palette.gray5,
        primary1_selected: palette.gray4,
    };

    Stylesheet {
        background_color: colors.background,
        input: text_input::Style {
            background: colors.primary1.into(),
            border: border::rounded(8),
            icon: colors.not_set,
            placeholder: colors.not_set,
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
                    color: colors.not_set,
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
            border: border::rounded(8),
            ..Default::default()
        },
        list_tab_selected: container::Style {
            background: Some(colors.primary1_selected.into()),
            border: border::rounded(8),
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
            placeholder: colors.not_set,
            value: colors.not_set,
            selection: colors.not_set,
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
