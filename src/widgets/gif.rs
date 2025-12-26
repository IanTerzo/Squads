use iced::advanced::image::{self, FilterMethod, Handle};
use iced::advanced::mouse::Cursor;
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, renderer};
use iced::border::Radius;
use iced::{ContentFit, Element, Event, Length, Point, Rectangle, Rotation, Size, Vector, window};
use image_rs::codecs::gif;
use image_rs::{AnimationDecoder, Frame, Frames};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
struct State<'a> {
    index: usize,
    frames: Frames<'a>,
    collecte_frames: Vec<Frame>,
    current: Current,
}

struct Current {
    frame: Frame,
    started: Instant,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Gif {
    path: PathBuf,
    width: Length,
    height: Length,
    crop: Option<Rectangle<u32>>,
    border_radius: Radius,
    content_fit: ContentFit,
    filter_method: FilterMethod,
    rotation: Rotation,
    opacity: f32,
    scale: f32,
    expand: bool,
}

impl<'a> Gif {
    /// Creates a new [`Gif`] with the given [`Frames`]
    pub fn new(path: PathBuf) -> Self {
        Gif {
            path: path,
            width: Length::Shrink,
            height: Length::Shrink,
            crop: None,
            border_radius: Radius::default(),
            content_fit: ContentFit::default(),
            filter_method: FilterMethod::default(),
            rotation: Rotation::default(),
            opacity: 1.0,
            scale: 1.0,
            expand: false,
        }
    }

    /// Sets the width of the [`Gif`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Gif`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`ContentFit`] of the [`Image`].
    ///
    /// Defaults to [`ContentFit::Contain`]
    pub fn content_fit(mut self, content_fit: ContentFit) -> Self {
        self.content_fit = content_fit;
        self
    }

    /// Sets the [`FilterMethod`] of the [`Image`].
    pub fn filter_method(mut self, filter_method: FilterMethod) -> Self {
        self.filter_method = filter_method;
        self
    }

    /// Applies the given [`Rotation`] to the [`Image`].
    pub fn rotation(mut self, rotation: impl Into<Rotation>) -> Self {
        self.rotation = rotation.into();
        self
    }

    /// Sets the opacity of the [`Image`].
    ///
    /// It should be in the [0.0, 1.0] range—`0.0` meaning completely transparent,
    /// and `1.0` meaning completely opaque.
    pub fn opacity(mut self, opacity: impl Into<f32>) -> Self {
        self.opacity = opacity.into();
        self
    }

    pub fn crop(mut self, region: Rectangle<u32>) -> Self {
        self.crop = Some(region);
        self
    }
}

fn crop(size: Size<u32>, region: Option<Rectangle<u32>>) -> Size<f32> {
    if let Some(region) = region {
        Size::new(
            region.width.min(size.width) as f32,
            region.height.min(size.height) as f32,
        )
    } else {
        Size::new(size.width as f32, size.height as f32)
    }
}

fn drawing_bounds<Renderer, Handle>(
    renderer: &Renderer,
    bounds: Rectangle,
    handle: &Handle,
    region: Option<Rectangle<u32>>,
    content_fit: ContentFit,
    rotation: Rotation,
    scale: f32,
) -> Rectangle
where
    Renderer: image::Renderer<Handle = Handle>,
{
    let original_size = renderer.measure_image(handle).unwrap_or_default();
    let image_size = crop(original_size, region);
    let rotated_size = rotation.apply(image_size);
    let adjusted_fit = content_fit.fit(rotated_size, bounds.size());

    let fit_scale = Vector::new(
        adjusted_fit.width / rotated_size.width,
        adjusted_fit.height / rotated_size.height,
    );

    let final_size = image_size * fit_scale * scale;

    let (crop_offset, final_size) = if let Some(region) = region {
        let x = region.x.min(original_size.width) as f32;
        let y = region.y.min(original_size.height) as f32;
        let width = image_size.width;
        let height = image_size.height;

        let ratio = Vector::new(
            original_size.width as f32 / width,
            original_size.height as f32 / height,
        );

        let final_size = final_size * ratio;

        let scale = Vector::new(
            final_size.width / original_size.width as f32,
            final_size.height / original_size.height as f32,
        );

        let offset = match content_fit {
            ContentFit::None => Vector::new(x * scale.x, y * scale.y),
            _ => Vector::new(
                ((original_size.width as f32 - width) / 2.0 - x) * scale.x,
                ((original_size.height as f32 - height) / 2.0 - y) * scale.y,
            ),
        };

        (offset, final_size)
    } else {
        (Vector::ZERO, final_size)
    };

    let position = match content_fit {
        ContentFit::None => Point::new(
            bounds.x + (rotated_size.width - adjusted_fit.width) / 2.0,
            bounds.y + (rotated_size.height - adjusted_fit.height) / 2.0,
        ),
        _ => Point::new(
            bounds.center_x() - final_size.width / 2.0,
            bounds.center_y() - final_size.height / 2.0,
        ),
    };

    Rectangle::new(position + crop_offset, final_size)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Gif
where
    Renderer: image::Renderer<Handle = Handle>,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let bytes = fs::read(self.path.clone()).unwrap();
        let decoder = gif::GifDecoder::new(io::Cursor::new(bytes)).unwrap();
        let mut frames = decoder.into_frames();

        let frame = frames.by_ref().nth(0).unwrap().unwrap();

        tree::State::new(State {
            index: 0,
            frames,
            collecte_frames: Vec::new(),
            current: Current {
                frame,
                started: Instant::now(),
            },
        })
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();

        let current = state.current.frame.clone();
        let (width, height) = current.buffer().dimensions();
        let handle = image::Handle::from_rgba(width, height, current.into_buffer().into_vec());

        // The raw w/h of the underlying image
        let image_size = crop(
            renderer.measure_image(&handle).unwrap_or_default(),
            self.crop,
        );

        // The rotated size of the image
        let rotated_size = self.rotation.apply(image_size);

        // The size to be available to the widget prior to `Shrink`ing
        let bounds = if self.expand {
            limits.width(width).height(height).max()
        } else {
            limits.resolve(width, height, rotated_size)
        };

        // The uncropped size of the image when fit to the bounds above
        let _full_size = self.content_fit.fit(rotated_size, bounds);

        // Shrink the widget to fit the resized image, if requested
        let final_size = Size {
            width: bounds.width,
            height: bounds.height,
        };

        layout::Node::new(final_size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let elapsed = now.duration_since(state.current.started);
            let delay: Duration = state.current.frame.delay().into();
            if elapsed > delay {
                // Take all the frames during the first run
                let next_frame = if let Some(frame_result) = state.frames.next() {
                    let frame = frame_result.unwrap();
                    state.collecte_frames.push(frame.clone());
                    frame
                } else {
                    state.index = (state.index + 1) % state.collecte_frames.len();
                    state.collecte_frames[state.index].clone()
                };

                let delay: Duration = next_frame.delay().into();

                state.current = Current {
                    frame: next_frame,
                    started: Instant::now(),
                };

                shell.request_redraw_at(window::RedrawRequest::At(*now + delay));
            } else {
                let remaining = delay - elapsed;
                shell.request_redraw_at(window::RedrawRequest::At(*now + remaining));
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        // Pulled from iced_native::widget::<Image as Widget>::draw
        //
        // TODO: export iced_native::widget::image::draw as standalone function
        {
            let frame = state.current.frame.clone();
            let (width, height) = frame.buffer().dimensions();
            let handle = image::Handle::from_rgba(width, height, frame.into_buffer().into_vec());

            let bounds = layout.bounds();

            let drawing_bounds = drawing_bounds(
                renderer,
                bounds,
                &handle,
                self.crop,
                self.content_fit,
                self.rotation,
                self.scale,
            );

            renderer.draw_image(
                image::Image {
                    handle,
                    filter_method: self.filter_method,
                    rotation: self.rotation.radians(),
                    opacity: self.opacity,
                    snap: true,
                    border_radius: 0.into(),
                },
                drawing_bounds,
                bounds,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Gif> for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = Handle> + 'a,
{
    fn from(gif: Gif) -> Element<'a, Message, Theme, Renderer> {
        Element::new(gif)
    }
}
