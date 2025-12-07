use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, tree, Tree, Widget};
use iced::{Element, Length, Rectangle, Size};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// TODO: Fix childrens children not being rendered when inside viewport widget
// This widget could probably be improved since i'm not very good with Iced.

static IDENTIFIERS: LazyLock<Mutex<HashMap<String, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct ViewportHandler<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    identifier: Option<String>,
    on_enter: Option<Message>,
}

impl<'a, Message, Theme, Renderer> ViewportHandler<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            identifier: None,
            on_enter: None,
        }
    }

    // Call a message once for all widgets with the identifier when the widget enters the viewport
    pub fn on_enter_unique(mut self, identifier: String, message: Message) -> Self {
        self.identifier = Some(identifier.clone());
        self.on_enter = Some(message);

        let mut identifiers = IDENTIFIERS.lock().unwrap();

        if !identifiers.contains_key(&identifier) {
            identifiers.insert(identifier, false);
        }

        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ViewportHandler<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self.content.as_widget_mut().layout(tree, renderer, limits);
        layout::Node::container(child, 0.into())
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let content_layout = layout.children().next().unwrap();
        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            viewport,
        );
    }
    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &iced::Event,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let relative_layout_position = layout.position().y;
        let viewport_top = viewport.y + viewport.height;

        if let Some(identfier) = self.identifier.clone() {
            let mut identifiers = IDENTIFIERS.lock().unwrap();
            let called = identifiers.get(&identfier).unwrap();
            if relative_layout_position < viewport_top && !called {
                if let Some(message) = self.on_enter.take() {
                    shell.publish(message);
                }
                identifiers.insert(identfier, true);
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ViewportHandler<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
{
    fn from(container: ViewportHandler<'a, Message, Theme, Renderer>) -> Self {
        Element::new(container)
    }
}
