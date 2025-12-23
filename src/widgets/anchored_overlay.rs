use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, overlay, renderer, widget};
use iced::{Element, Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse};

use crate::Theme;

pub fn anchored_overlay<'a, Message: 'a>(
    base: impl Into<Element<'a, Message>>,
    overlay: impl Into<Element<'a, Message>>,
    anchor: Position,
    offset: f32,
    visible: bool,
    window_size: (f32, f32),
) -> Element<'a, Message> {
    AnchoredOverlay {
        base: base.into(),
        overlay: overlay.into(),
        anchor,
        offset,
        visible,
        window_size: window_size,
    }
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

struct AnchoredOverlay<'a, Message> {
    base: Element<'a, Message>,
    overlay: Element<'a, Message>,
    anchor: Position,
    offset: f32,
    visible: bool,
    window_size: (f32, f32),
}

impl<Message> Widget<Message, Theme, Renderer> for AnchoredOverlay<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.base.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.base),
            widget::Tree::new(&self.overlay),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.base, &self.overlay]);
    }

    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.base
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let (first, second) = tree.children.split_at_mut(1);

        let base = self.base.as_widget_mut().overlay(
            &mut first[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let overlay = if self.visible {
            Some(overlay::Element::new(Box::new(Overlay {
                content: &mut self.overlay,
                tree: &mut second[0],
                anchor: self.anchor,
                offset: self.offset,
                base_layout: layout.bounds(),
                position: layout.position() + translation,
                viewport: *viewport,
                window_size: self.window_size,
            })))
        } else {
            None
        };

        if base.is_some() || overlay.is_some() {
            Some(overlay::Group::with_children(base.into_iter().chain(overlay).collect()).overlay())
        } else {
            None
        }
    }
}

impl<'a, Message> From<AnchoredOverlay<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(anchored_overlay: AnchoredOverlay<'a, Message>) -> Self {
        Element::new(anchored_overlay)
    }
}

struct Overlay<'a, 'b, Message> {
    content: &'b mut Element<'a, Message>,
    tree: &'b mut widget::Tree,
    anchor: Position,
    offset: f32,
    base_layout: Rectangle,
    position: Point,
    viewport: Rectangle,
    window_size: (f32, f32),
}

impl<Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = match self.anchor {
            Position::Top => layout::Limits::new(
                Size::ZERO,
                Size {
                    width: bounds.width,
                    height: self.position.y, // space above the base
                },
            ),
            Position::Bottom => layout::Limits::new(
                Size::ZERO,
                Size {
                    width: bounds.width,
                    height: bounds.height - self.position.y - self.base_layout.height, // space below
                },
            ),
            Position::Left => layout::Limits::new(
                Size::ZERO,
                Size {
                    width: self.position.x, // space to the left
                    height: bounds.height,
                },
            ),
            Position::Right => layout::Limits::new(
                Size::ZERO,
                Size {
                    width: bounds.width - self.position.x - self.base_layout.width, // space to the right
                    height: bounds.height,
                },
            ),
        };

        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        let translation = match self.anchor {
            Position::Top => Vector::new(0.0, -(node.size().height + self.offset)),
            Position::Bottom => Vector::new(0.0, self.base_layout.height + self.offset),
            Position::Left => Vector::new(-(node.size().width + self.offset), 0.0),
            Position::Right => Vector::new(self.base_layout.width + self.offset, 0.0),
        };

        let mut desired = self.position + translation;

        let (window_width, window_height) = self.window_size;
        let padding = 20.0;

        // Compute padded window bounds
        let min_x = padding;
        let min_y = padding;

        let max_x = (window_width - node.size().width - padding).max(min_x);
        let max_y = (window_height - node.size().height - padding).max(min_y);

        // Clamp to padded window
        desired.x = desired.x.clamp(min_x, max_x);
        desired.y = desired.y.clamp(min_y, max_y);

        node.move_to(desired)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
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
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
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
