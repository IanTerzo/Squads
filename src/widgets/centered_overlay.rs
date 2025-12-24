use iced::advanced::Renderer as _;
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, overlay, renderer, widget};
use iced::{Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse};

use crate::Theme;

pub fn centered_overlay<'a, Message: 'a>(
    overlay: impl Into<Element<'a, Message>>,
    window_size: (f32, f32),
    background_opacity: f32,
) -> Element<'a, Message> {
    CenteredOverlay {
        overlay: overlay.into(),
        window_size,
        background_opacity,
    }
    .into()
}

struct CenteredOverlay<'a, Message> {
    overlay: Element<'a, Message>,
    window_size: (f32, f32),
    background_opacity: f32,
}

impl<Message> Widget<Message, Theme, Renderer> for CenteredOverlay<'_, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.overlay)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.overlay]);
    }

    fn operate(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn widget::Operation<()>,
    ) {
    }

    fn update(
        &mut self,
        _tree: &mut widget::Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        _layout: Layout<'b>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        Some(overlay::Element::new(Box::new(Overlay {
            content: &mut self.overlay,
            tree: &mut tree.children[0],
            viewport: *viewport,
            window_size: self.window_size,
            background_opacity: self.background_opacity,
        })))
    }
}

impl<'a, Message> From<CenteredOverlay<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(anchored_overlay: CenteredOverlay<'a, Message>) -> Self {
        Element::new(anchored_overlay)
    }
}

struct Overlay<'a, 'b, Message> {
    content: &'b mut Element<'a, Message>,
    tree: &'b mut widget::Tree,
    viewport: Rectangle,
    window_size: (f32, f32),
    background_opacity: f32,
}

impl<Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, Size::INFINITE);

        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        let (window_width, window_height) = self.window_size;
        let padding = 20.0;

        let x = (window_width - node.size().width) / 2.0;
        let y = (window_height - node.size().height) / 2.0;

        let min_x = padding;
        let min_y = padding;
        let max_x = (window_width - node.size().width - padding).max(min_x);
        let max_y = (window_height - node.size().height - padding).max(min_y);

        let position = Point::new(x.clamp(min_x, max_x), y.clamp(min_y, max_y));

        node.move_to(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        // Draw dimmed background
        if self.background_opacity > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: self.viewport.width,
                        height: self.viewport.height,
                    },
                    ..renderer::Quad::default()
                },
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: self.background_opacity.clamp(0.0, 1.0),
                },
            );
        }

        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );

        if matches!(
            event,
            Event::Mouse(_) | Event::Touch(_) | Event::Keyboard(_)
        ) {
            shell.invalidate_layout();
        }
    }

    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        if cursor.is_over(self.viewport) {
            iced::advanced::mouse::Interaction::Idle
        } else {
            iced::advanced::mouse::Interaction::default()
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &self.viewport,
            Vector::default(),
        )
    }
}
