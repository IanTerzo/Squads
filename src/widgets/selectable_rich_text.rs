use std::borrow::Cow;
use std::sync::Arc;

use iced::advanced::graphics::core::touch;
use iced::advanced::renderer::Quad;
use iced::advanced::text::{self, Highlight, Paragraph, Span, Text};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::Operation;
use iced::advanced::{layout, renderer, Clipboard, Layout, Shell, Widget};
use iced::widget::container;
use iced::widget::text::{LineHeight, Shaping};
use iced::widget::text_input::Value;
use iced::{
    self, alignment, event, mouse, widget, Alignment, Background, Border, Color, Element, Event,
    Length, Pixels, Point, Rectangle, Shadow, Size, Vector,
};
use itertools::Itertools;
use super::selectable_text::{selection, Catalog, Interaction, Style, StyleFn};

/// Creates a new [`Rich`] text widget with the provided spans.
pub fn selectable_rich_text<'a, Message, Link, Entry, Theme, Renderer>(
    spans: impl Into<Cow<'a, [Span<'a, Link, Renderer::Font>]>>,
) -> Rich<'a, Message, Link, Entry, Theme, Renderer>
where
    Link: self::Link + 'static,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    Rich::with_spans(spans)
}

/// A bunch of [`Rich`] text.
#[allow(missing_debug_implementations)]
pub struct Rich<'a, Message, Link = (), Entry = (), Theme = iced::Theme, Renderer = iced::Renderer>
where
    Link: self::Link + 'static,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    spans: Cow<'a, [Span<'a, Link, Renderer::Font>]>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    font: Option<Renderer::Font>,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    class: Theme::Class<'a>,
    on_link: Option<Box<dyn Fn(Link) -> Message + 'a>>,

    #[allow(clippy::type_complexity)]
    cached_entries: Vec<Entry>,
    cached_menu: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Link, Entry, Theme, Renderer> Rich<'a, Message, Link, Entry, Theme, Renderer>
where
    Link: self::Link + 'static,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new empty [`Rich`] text.
    pub fn new() -> Self {
        Self {
            spans: Cow::default(),
            size: None,
            line_height: LineHeight::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            font: None,
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Top,
            class: Theme::default(),
            on_link: None,

            cached_entries: vec![],
            cached_menu: None,
        }
    }

    /// Creates a new [`Rich`] text with the given text spans.
    pub fn with_spans(spans: impl Into<Cow<'a, [Span<'a, Link, Renderer::Font>]>>) -> Self {
        Self {
            spans: spans.into(),
            ..Self::new()
        }
    }

    /// Sets the default size of the [`Rich`] text.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the default [`LineHeight`] of the [`Rich`] text.
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the default font of the [`Rich`] text.
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the width of the [`Rich`] text boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Rich`] text boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Centers the [`Rich`] text, both horizontally and vertically.
    pub fn center(self) -> Self {
        self.align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
    }

    /// Sets the [`alignment::Horizontal`] of the [`Rich`] text.
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Rich`] text.
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();
        self
    }

    /// Sets the default style of the [`Rich`] text.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the default [`Color`] of the [`Rich`] text.
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    /// Sets the default [`Color`] of the [`Rich`] text, if `Some`.
    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| Style {
            color,
            selection_color: Color::WHITE,
        })
    }

    /// Sets the message handler for link clicks on the [`Rich`] text.
    pub fn on_link(mut self, on_link: impl Fn(Link) -> Message + 'a) -> Self {
        self.on_link = Some(Box::new(on_link));
        self
    }
}

impl<Message, Link, Entry, Theme, Renderer> Default
    for Rich<'_, Message, Link, Entry, Theme, Renderer>
where
    Link: self::Link + 'static,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait Link: Clone {
    fn underline(&self) -> bool {
        true
    }
}

impl Link for () {}

struct State<Link, P: Paragraph> {
    spans: Vec<Span<'static, Link, P::Font>>,
    span_pressed: Option<usize>,
    paragraph: P,
    hovered: bool,
    link_hovered: bool,
    interaction: Interaction,
    shown_spoiler: Option<(usize, Color, Highlight)>,
}

struct Snapshot {
    hovered: bool,
    link_hovered: bool,
    span_pressed: Option<usize>,
    interaction: Interaction,
    shown_spoiler: Option<(usize, Color, Highlight)>,
}

impl<Link, P: Paragraph> From<&State<Link, P>> for Snapshot {
    fn from(value: &State<Link, P>) -> Self {
        Snapshot {
            hovered: value.hovered,
            link_hovered: value.link_hovered,
            span_pressed: value.span_pressed,
            interaction: value.interaction,
            shown_spoiler: value.shown_spoiler,
        }
    }
}

impl Snapshot {
    fn is_changed(&self, other: &Self) -> bool {
        self.hovered != other.hovered
            || self.link_hovered != other.link_hovered
            || self.span_pressed != other.span_pressed
            || self.interaction != other.interaction
            || self.shown_spoiler != other.shown_spoiler
    }
}

impl<'a, Message, Link, Entry, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Rich<'a, Message, Link, Entry, Theme, Renderer>
where
    Message: 'a,
    Link: self::Link + 'static,
    Entry: Copy + 'a,
    Theme: 'a + container::Catalog + Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    Renderer: text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Link, Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Link, _> {
            spans: Vec::new(),
            span_pressed: None,
            paragraph: Renderer::Paragraph::default(),
            interaction: Interaction::default(),
            shown_spoiler: None,
            hovered: false,
            link_hovered: false,
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            tree.state
                .downcast_mut::<State<Link, Renderer::Paragraph>>(),
            renderer,
            limits,
            self.width,
            self.height,
            self.spans.as_ref(),
            self.line_height,
            self.size,
            self.font,
            self.align_x.into(),
            self.align_y,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let state = tree
            .state
            .downcast_mut::<State<Link, Renderer::Paragraph>>();
        let prev_snapshot = Snapshot::from(&*state);

        let bounds = layout.bounds();

        if viewport.intersection(&bounds).is_none()
            && matches!(state.interaction, Interaction::Idle)
        {
            return event::Status::Ignored;
        }

        state.hovered = false;
        state.link_hovered = false;

        if let Some(position) = cursor.position_in(layout.bounds()) {
            state.hovered = true;

            if self.on_link.is_some() {
                if let Some(span) = state
                    .paragraph
                    .hit_span(position)
                    .and_then(|span| self.spans.get(span))
                {
                    if span.link.is_some() {
                        state.link_hovered = true;
                    }
                }
            }
        }

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(position) = cursor.position_in(bounds) {
                    if let Some(span) = state.paragraph.hit_span(position) {
                        state.span_pressed = Some(span);
                    }
                }

                if let Some(cursor) = cursor.position() {
                    state.interaction = Interaction::Selecting(selection::Raw {
                        start: cursor,
                        end: cursor,
                    });
                } else {
                    state.interaction = Interaction::Idle;
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerLifted { .. })
            | iced::Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some(on_link_click) = self.on_link.as_ref() {
                    if let Some(span_pressed) = state.span_pressed {
                        state.span_pressed = None;

                        if let Some(position) = cursor.position_in(bounds) {
                            match state.paragraph.hit_span(position) {
                                Some(span) if span == span_pressed => {
                                    if let Some(link) =
                                        self.spans.get(span).and_then(|span| span.link.clone())
                                    {
                                        shell.publish(on_link_click(link));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if let Interaction::Selecting(raw) = state.interaction {
                    state.interaction = Interaction::Selected(raw);
                } else {
                    state.interaction = Interaction::Idle;
                }
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. })
            | iced::Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(cursor) = cursor.position() {
                    if let Interaction::Selecting(raw) = &mut state.interaction {
                        raw.end = cursor;
                    }
                }

                let size = self.size.unwrap_or_else(|| renderer.default_size());
                let font = self.font.unwrap_or_else(|| renderer.default_font());

                let text_with_spans = |spans| Text {
                    content: spans,
                    bounds: bounds.size(),
                    size,
                    line_height: self.line_height,
                    font,
                    horizontal_alignment: self.align_x,
                    vertical_alignment: self.align_y,
                    shaping: Shaping::Advanced,
                    wrapping: text::Wrapping::WordOrGlyph,
                };

                // Check spoiler
                if let Some(cursor) = cursor.position_in(bounds) {
                    if state.shown_spoiler.is_none() {
                        // Find if spoiler is hovered
                        for (index, span) in state.spans.iter().enumerate() {
                            if let Some((fg, highlight)) = span.color.zip(span.highlight) {
                                let is_spoiler = highlight.background == Background::Color(fg);

                                if is_spoiler
                                    && state
                                        .paragraph
                                        .span_bounds(index)
                                        .into_iter()
                                        .any(|bounds| bounds.contains(cursor))
                                {
                                    state.shown_spoiler = Some((index, fg, highlight));
                                    break;
                                }
                            }
                        }

                        // Show spoiler
                        if let Some((index, _, _)) = state.shown_spoiler {
                            // Safe we just got this index
                            let span = &mut state.spans[index];
                            span.color = None;
                            span.highlight = None;
                            state.paragraph = Renderer::Paragraph::with_spans(text_with_spans(
                                state.spans.as_ref(),
                            ));
                        }
                    }
                }
                // Hide spoiler
                else if let Some((index, fg, highlight)) = state.shown_spoiler.take() {
                    if let Some(span) = state.spans.get_mut(index) {
                        span.color = Some(fg);
                        span.highlight = Some(highlight);
                    }
                    state.paragraph =
                        Renderer::Paragraph::with_spans(text_with_spans(state.spans.as_ref()));
                }
            }

            _ => {}
        }

        if prev_snapshot.is_changed(&Snapshot::from(&*state)) {
            shell.request_redraw(iced::window::RedrawRequest::NextFrame);
        }

        event::Status::Captured
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if viewport.intersection(&bounds).is_none() {
            return;
        }

        let state = tree
            .state
            .downcast_ref::<State<Link, Renderer::Paragraph>>();

        let style = <Theme as Catalog>::style(theme, &self.class);

        let hovered_span = cursor
            .position_in(layout.bounds())
            .and_then(|position| state.paragraph.hit_span(position));

        for (index, span) in state.spans.iter().enumerate() {
            let is_hovered_link = span.link.is_some() && Some(index) == hovered_span;

            if span.highlight.is_some() || span.underline || span.strikethrough || is_hovered_link {
                let translation = layout.position() - Point::ORIGIN;
                let regions = state.paragraph.span_bounds(index);

                if let Some(highlight) = span.highlight {
                    for bounds in &regions {
                        let bounds = Rectangle::new(
                            bounds.position() - Vector::new(span.padding.left, span.padding.top),
                            bounds.size()
                                + Size::new(span.padding.horizontal(), span.padding.vertical()),
                        );

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: bounds + translation,
                                border: highlight.border,
                                ..Default::default()
                            },
                            highlight.background,
                        );
                    }
                }

                if span.underline || span.strikethrough || is_hovered_link {
                    let size = span.size.or(self.size).unwrap_or(renderer.default_size());

                    let line_height = span
                        .line_height
                        .unwrap_or(self.line_height)
                        .to_absolute(size);

                    let color = span.color.or(style.color).unwrap_or(defaults.text_color);

                    let baseline =
                        translation + Vector::new(0.0, size.0 + (line_height.0 - size.0) / 2.0);

                    if span.underline
                        || (is_hovered_link && span.link.as_ref().unwrap().underline())
                    {
                        for bounds in &regions {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        bounds.position() + baseline
                                            - Vector::new(0.0, size.0 * 0.08),
                                        Size::new(bounds.width, 1.0),
                                    ),
                                    ..Default::default()
                                },
                                color,
                            );
                        }
                    }

                    if span.strikethrough {
                        for bounds in &regions {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        bounds.position() + baseline
                                            - Vector::new(0.0, size.0 / 2.0),
                                        Size::new(bounds.width, 1.0),
                                    ),
                                    ..Default::default()
                                },
                                color,
                            );
                        }
                    }
                }
            }
        }

        if let Some(selection) = state
            .interaction
            .selection()
            .and_then(|raw| raw.resolve(bounds))
        {
            let line_height = f32::from(
                self.line_height
                    .to_absolute(self.size.unwrap_or_else(|| renderer.default_size())),
            );

            let baseline_y =
                bounds.y + ((selection.start.y - bounds.y) / line_height).floor() * line_height;

            let height = selection.end.y - baseline_y - 0.5;
            let rows = (height / line_height).ceil() as usize;

            for row in 0..rows {
                let (x, width) = if row == 0 {
                    (
                        selection.start.x,
                        if rows == 1 {
                            f32::min(selection.end.x, bounds.x + bounds.width) - selection.start.x
                        } else {
                            bounds.x + bounds.width - selection.start.x
                        },
                    )
                } else if row == rows - 1 {
                    (bounds.x, selection.end.x - bounds.x)
                } else {
                    (bounds.x, bounds.width)
                };
                let y = baseline_y + row as f32 * line_height;

                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(Point::new(x, y), Size::new(width, line_height)),
                        border: Border {
                            radius: 0.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow::default(),
                    },
                    style.selection_color,
                );
            }
        }

        widget::text::draw(
            renderer,
            defaults,
            layout,
            &state.paragraph,
            widget::text::Style { color: style.color },
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree
            .state
            .downcast_ref::<State<Link, Renderer::Paragraph>>();

        if state.hovered {
            if state.link_hovered {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::None
        }
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        let state = tree
            .state
            .downcast_mut::<State<Link, Renderer::Paragraph>>();

        let bounds = layout.bounds();
        let value = Value::new(&self.spans.iter().map(|s| s.text.as_ref()).join(""));
        if let Some(selection) = state
            .interaction
            .selection()
            .and_then(|raw| selection(raw, bounds, &state.paragraph, &value))
        {
            let mut content = value.select(selection.start, selection.end).to_string();
            operation.custom(&mut content, None);
        }

        // Context menu
    }
}

fn layout<Link, Renderer>(
    state: &mut State<Link, Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    spans: &[Span<'_, Link, Renderer::Font>],
    line_height: LineHeight,
    size: Option<Pixels>,
    font: Option<Renderer::Font>,
    align_x: Alignment,
    align_y: alignment::Vertical,
) -> layout::Node
where
    Link: Clone,
    Renderer: text::Renderer,
{
    layout::sized(limits, width, height, |limits| {
        let bounds = limits.max();

        let size = size.unwrap_or_else(|| renderer.default_size());
        let font = font.unwrap_or_else(|| renderer.default_font());

        let text_with_spans = |spans| Text {
            content: spans,
            bounds,
            size,
            line_height,
            font,
            horizontal_alignment: align_x.into(),
            vertical_alignment: align_y,
            shaping: Shaping::Advanced,
            wrapping: text::Wrapping::WordOrGlyph,
        };

        if state.spans != spans {
            state.spans = spans.iter().cloned().map(Span::to_static).collect();

            // Apply shown spoiler
            if let Some((index, _, _)) = state.shown_spoiler {
                if let Some(span) = state.spans.get_mut(index) {
                    span.color = None;
                    span.highlight = None;
                }
            }

            state.paragraph =
                Renderer::Paragraph::with_spans(text_with_spans(state.spans.as_slice()));
        } else {
            match state.paragraph.compare(Text {
                content: (),
                bounds,
                size,
                line_height,
                font,
                horizontal_alignment: align_x.into(),
                vertical_alignment: align_y,
                shaping: Shaping::Advanced,
                wrapping: text::Wrapping::WordOrGlyph,
            }) {
                text::Difference::None => {}
                text::Difference::Bounds => {
                    state.paragraph.resize(bounds);
                }
                text::Difference::Shape => {
                    state.spans = spans.iter().cloned().map(Span::to_static).collect();

                    // Apply shown spoiler
                    if let Some((index, _, _)) = state.shown_spoiler {
                        if let Some(span) = state.spans.get_mut(index) {
                            span.color = None;
                            span.highlight = None;
                        }
                    }

                    state.paragraph =
                        Renderer::Paragraph::with_spans(text_with_spans(state.spans.as_slice()));
                }
            }
        }

        state.paragraph.min_bounds()
    })
}

impl<'a, Message, Link, Entry, Theme, Renderer> FromIterator<Span<'a, Link, Renderer::Font>>
    for Rich<'a, Message, Link, Entry, Theme, Renderer>
where
    Link: self::Link + 'static,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn from_iter<T: IntoIterator<Item = Span<'a, Link, Renderer::Font>>>(spans: T) -> Self {
        Self {
            spans: spans.into_iter().collect(),
            ..Self::new()
        }
    }
}

impl<'a, Message, Link, Entry, Theme, Renderer>
    From<Rich<'a, Message, Link, Entry, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Link: self::Link + 'static,
    Entry: Copy + 'a,
    Theme: 'a + container::Catalog + Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    Renderer: text::Renderer + 'a,
{
    fn from(
        text: Rich<'a, Message, Link, Entry, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}
