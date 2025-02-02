use crate::api::{Channel, Team, TeamConversations};
use crate::Message;

use iced::font::Weight;
use iced::widget::{
    column, container, image, rich_text, row, scrollable, svg, text, text::Rich, text::Span,
    text_input, Column, Image, MouseArea, Row, Space, TextInput,
};
use iced::{border, font, padding, Color, ContentFit, Element, Font, Padding};
use iced_widget::Container;

use std::path::Path;

mod navbar;
use navbar::navbar;

mod viewport;
use viewport::ViewportHandler;

use scraper::{Html, Selector};

enum DynamicContainer {
    Row(Row<'static, Message>),
    RowWrapping(Container<'static, Message>), // I must put it in a container since iced_widget:row:Wrapping is not public
    Column(Column<'static, Message>),
}

impl DynamicContainer {
    fn push(self, element: Element<'static, Message>) -> Self {
        match self {
            DynamicContainer::Row(mut row) => {
                row = row.push(element);
                DynamicContainer::Row(row)
            }
            DynamicContainer::RowWrapping(row_container) => {
                DynamicContainer::RowWrapping(row_container)
            }
            DynamicContainer::Column(mut column) => {
                column = column.push(element);
                DynamicContainer::Column(column)
            }
        }
    }

    fn into_element(self) -> Element<'static, Message> {
        match self {
            DynamicContainer::Row(row) => row.into(),
            DynamicContainer::RowWrapping(row_container) => row_container.into(),
            DynamicContainer::Column(column) => column.into(),
        }
    }

    fn wrap(self) -> Self {
        match self {
            DynamicContainer::Row(row) => {
                let wrapped = row.wrap(); // `row.wrap()` moves `row`, so capture the result
                DynamicContainer::RowWrapping(container(wrapped))
            }
            DynamicContainer::RowWrapping(row_container) => {
                DynamicContainer::RowWrapping(row_container)
            }

            DynamicContainer::Column(column) => DynamicContainer::Column(column),
        }
    }
}

fn transform_html(
    element: scraper::ElementRef,
    mut cascading_properties: Vec<String>,
) -> DynamicContainer {
    let element_tagname = element.value().name();

    // Initialize container as either row or column based on the tag name
    let mut container = if element_tagname == "body" || element_tagname == "div" {
        DynamicContainer::Column(column![])
    } else if element_tagname == "p" {
        DynamicContainer::Row(row![])
    } else {
        // Default to a row if no specific tag matches
        DynamicContainer::Row(row![])
    };

    if matches!(element_tagname, "strong" | "u" | "s") {
        cascading_properties.push(element_tagname.to_string());
    }

    for child in element.children() {
        if child.has_children() {
            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                container = container.push(
                    transform_html(child_element, cascading_properties.clone()).into_element(),
                );
            }
        } else if child.value().is_text() {
            let text_content = child.value().as_text().unwrap().text.to_string();

            let words = text_content.split_inclusive(" ");

            // Turn every word into its own rich text (not ideal but necessary to mantain correct wrapping)
            for word in words {
                let mut text_span = Span::new(word.to_string());

                for property in &cascading_properties {
                    if property == "strong" {
                        let font_bold = Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        };
                        text_span = text_span.font(font_bold);
                    }
                    if property == "s" {
                        text_span = text_span.strikethrough(true);
                    }
                    if property == "u" {
                        text_span = text_span.underline(true);
                    }
                }

                // Wrap in rich_text and push to container
                let text = rich_text![text_span];
                container = container.push(text.into());
            }
        }
    }

    container.wrap() // If it is a row, it needs to be wrapping
}

fn parse_message_html(content: String) -> Element<'static, Message> {
    // Remove things like newlines to avoid them being treated as text during the parsing
    let content = content.replace("\n", "").replace("\r", "");
    let document = Html::parse_document(content.as_str());

    let selector = Selector::parse("body").unwrap();
    if let Some(root_element) = document.select(&selector).next() {
        transform_html(root_element, vec![]).into_element()
    } else {
        text("").into()
    }
}

// helper functions
fn truncate_name(name: &str, max_length: usize) -> String {
    if name.len() > max_length {
        let mut truncated = name.to_string();
        truncated.replace_range(max_length - 3.., "...");
        truncated
    } else {
        name.to_string()
    }
}

// UI functions
pub fn app<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    column![navbar(), container(content).padding(20)].into()
}

pub fn homepage<'a>(teams: Vec<Team>, search_teams_input_value: String) -> Element<'a, Message> {
    let mut teams_column: Column<Message> = column![];

    for team in teams {
        let mut team_picture = container(ViewportHandler::new(Space::new(0, 0)).on_enter_unique(
            team.id.clone(),
            Message::FetchTeamImage(
                team.picture_e_tag.clone(),
                team.team_site_information.group_id.clone(),
                team.display_name.clone(),
            ),
        ))
        .style(|_| container::Style {
            background: Some(
                Color::parse("#b8b4b4")
                    .expect("Background color is invalid.")
                    .into(),
            ),

            ..Default::default()
        })
        .height(28)
        .width(28);

        let image_path = format!("image-cache/{}.jpeg", team.picture_e_tag);

        if Path::new(&image_path).exists() {
            team_picture = container(
                ViewportHandler::new(
                    image(image_path)
                        .content_fit(ContentFit::Cover)
                        .width(28)
                        .height(28),
                )
                .on_enter_unique(
                    team.id.clone(),
                    Message::FetchTeamImage(
                        team.picture_e_tag,
                        team.team_site_information.group_id,
                        team.display_name.clone(),
                    ),
                ),
            )
            .height(28)
            .width(28)
        }

        teams_column = teams_column.push(
            MouseArea::new(
                container(
                    row![
                        container(team_picture).padding(padding::left(10)),
                        text(truncate_name(&team.display_name, 16)),
                    ]
                    .spacing(10),
                )
                .style(|_| container::Style {
                    background: Some(
                        Color::parse("#333")
                            .expect("Background color is invalid.")
                            .into(),
                    ),
                    border: border::rounded(8),
                    ..Default::default()
                })
                .center_y(47)
                .width(220),
            )
            .on_press(Message::OpenTeam(
                team.id.clone().to_string(),
                team.id.to_string(),
            )),
        );
        teams_column = teams_column.push(Space::new(10, 8.5));
    }

    let team_scrollbar = container(
        scrollable(teams_column)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(10)
                    .spacing(10)
                    .scroller_width(10),
            ))
            .style(|_, _| scrollable::Style {
                container: container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0).into()),
                    border: border::rounded(10),
                    ..Default::default()
                },
                vertical_rail: scrollable::Rail {
                    background: Some(
                        Color::parse("#333")
                            .expect("Background color is invalid.")
                            .into(),
                    ),
                    border: border::rounded(10),
                    scroller: scrollable::Scroller {
                        color: Color::parse("#444").expect("Background color is invalid."),
                        border: border::rounded(10),
                    },
                },
                horizontal_rail: scrollable::Rail {
                    background: Some(
                        Color::parse("#333")
                            .expect("Background color is invalid.")
                            .into(),
                    ),
                    border: border::rounded(10),
                    scroller: scrollable::Scroller {
                        color: Color::parse("#666").expect("Background color is invalid."),
                        border: border::rounded(10),
                    },
                },
                gap: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
            }),
    );
    //.padding(20);

    let search_teams = container(
        text_input("Search teams.. .", &search_teams_input_value)
            .on_input(Message::ContentChanged)
            .padding(8)
            .style(|_, _| text_input::Style {
                background: Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
                border: border::rounded(8),
                icon: Color::parse("#444").expect("Background color is invalid."),
                placeholder: Color::parse("#666").expect("Background color is invalid."),
                value: Color::parse("#fff").expect("Background color is invalid."),
                selection: Color::parse("#444").expect("Background color is invalid."),
            }),
    )
    .width(220)
    .padding(padding::bottom(18));

    column![search_teams, team_scrollbar].into()
}

pub fn login<'a>() -> Element<'a, Message> {
    text("Sign in to your account on the browser window").into()
}

pub fn team_page<'a>(
    team: Team,
    page_channel: Channel,
    conversations: Option<TeamConversations>,
) -> Element<'a, Message> {
    let mut conversation_column = column![].spacing(10);

    if let Some(conversations) = conversations {
        let ordered_conversations: Vec<_> =
            conversations.reply_chains.iter().rev().cloned().collect();

        for conversation in ordered_conversations {
            let mut message_chain = column![];

            let ordered_messages: Vec<_> = conversation.messages.iter().rev().cloned().collect();

            for message in ordered_messages {
                if !message.properties.systemdelete {
                    let mut message_column = column![].padding(20).spacing(20);
                    if message.message_type == "RichText/Html" {
                        let im_display_name = message.im_display_name.unwrap();
                        let user_id = message.from.unwrap();

                        let mut user_picture =
                            container(ViewportHandler::new(Space::new(0, 0)).on_enter_unique(
                                user_id.clone(),
                                Message::FetchUserImage(user_id.clone(), im_display_name.clone()),
                            ))
                            .style(|_| container::Style {
                                background: Some(
                                    Color::parse("#b8b4b4")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),

                                ..Default::default()
                            })
                            .height(31)
                            .width(31);

                        let image_path = format!("image-cache/user-{}.jpeg", user_id.clone());

                        if Path::new(&image_path).exists() {
                            user_picture = container(
                                ViewportHandler::new(
                                    image(image_path)
                                        .content_fit(ContentFit::Cover)
                                        .width(31)
                                        .height(31),
                                )
                                .on_enter_unique(
                                    user_id.clone(),
                                    Message::FetchUserImage(
                                        user_id.clone(),
                                        im_display_name.clone(),
                                    ),
                                ),
                            )
                            .height(31)
                            .width(31)
                        }
                        let message_info = row![user_picture, text!("{}", im_display_name)]
                            .spacing(10)
                            .wrap();

                        message_column = message_column.push(message_info);
                    }

                    if message.properties.subject != "".to_string() {
                        message_column = message_column.push(
                            text(message.properties.subject).size(18).font(font::Font {
                                weight: font::Weight::Bold,
                                ..Default::default()
                            }),
                        );
                    }
                    if message.message_type == "RichText/Html" {
                        if let Some(content) = message.content {
                            message_column = message_column.push(parse_message_html(content));
                        }
                    } else {
                        if let Some(content) = message.content {
                            message_column = message_column.push(text(content));
                        }
                    }
                    message_chain = message_chain.push(message_column);
                }
            }
            conversation_column = conversation_column.push(
                container(message_chain)
                    .style(|_| container::Style {
                        background: Some(
                            Color::parse("#333")
                                .expect("Background color is invalid.")
                                .into(),
                        ),
                        border: border::rounded(8),
                        ..Default::default()
                    })
                    .width(iced::Length::Fill),
            );
        }
    }

    // TODO make it into a component
    let conversation_scrollbar = scrollable(conversation_column)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(10)
                .spacing(10)
                .scroller_width(10),
        ))
        .style(|_, _| scrollable::Style {
            container: container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0).into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#444").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#666").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            gap: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
        });

    let image_path = format!("image-cache/{}.jpeg", team.picture_e_tag);

    let team_picture = image(image_path)
        .content_fit(ContentFit::Cover)
        .width(45)
        .height(45);

    let name_row = row![
        team_picture,
        column![
            text!("{}", truncate_name(&team.display_name, 16)).font(font::Font {
                weight: font::Weight::Bold,
                ..Default::default()
            }),
            text!("{}", truncate_name(&page_channel.display_name, 16))
        ]
        .spacing(5)
    ]
    .spacing(10);

    let sidetabs = column![text!("Class Notebook"), text!("Assignments")].spacing(8);

    let mut channels_coloumn: Column<Message> = column![];

    let channel_count = team.channels.len();

    for channel in team.channels.clone() {
        let page_channel_cloned = page_channel.clone();
        let channel_cloned = channel.clone();
        channels_coloumn = channels_coloumn.push(
            MouseArea::new(
                container(text(truncate_name(&channel.display_name, 16)))
                    .style(move |_| {
                        if channel_cloned.id == page_channel_cloned.id {
                            container::Style {
                                background: Some(
                                    Color::parse("#4c4c4c")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(8),
                                ..Default::default()
                            }
                        } else {
                            container::Style {
                                background: Some(
                                    Color::parse("#333")
                                        .expect("Background color is invalid.")
                                        .into(),
                                ),
                                border: border::rounded(8),
                                ..Default::default()
                            }
                        }
                    })
                    .padding(Padding::from([0, 8]))
                    .center_y(47)
                    .width(if channel_count <= 13 { 220 } else { 185 }),
            )
            .on_press(Message::OpenTeam(team.clone().id, channel.id)),
        );
        channels_coloumn = channels_coloumn.push(Space::new(10, 8.5));
    }

    let team_scrollbar = scrollable(channels_coloumn)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(10)
                .spacing(10)
                .scroller_width(10),
        ))
        .style(|_, _| scrollable::Style {
            container: container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0).into()),
                border: border::rounded(10),
                ..Default::default()
            },
            vertical_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#444").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(
                    Color::parse("#333")
                        .expect("Background color is invalid.")
                        .into(),
                ),
                border: border::rounded(10),
                scroller: scrollable::Scroller {
                    color: Color::parse("#666").expect("Background color is invalid."),
                    border: border::rounded(10),
                },
            },
            gap: Some(
                Color::parse("#333")
                    .expect("Background color is invalid.")
                    .into(),
            ),
        });

    let team_info_column = column![name_row, sidetabs, team_scrollbar].spacing(18);
    row![team_info_column, conversation_scrollbar]
        .spacing(10)
        .into()
}
