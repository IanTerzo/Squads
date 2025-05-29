use iced::advanced::text;
use iced::{Point, Rectangle, Vector};

use super::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Raw {
    pub start: Point,
    pub end: Point,
}

impl Raw {
    pub fn resolve(&self, bounds: Rectangle) -> Option<Resolved> {
        if f32::max(f32::min(self.start.y, self.end.y), bounds.y)
            <= f32::min(
                f32::max(self.start.y, self.end.y),
                bounds.y + bounds.height,
            )
        {
            let (mut start, mut end) = if self.start.y < self.end.y
                || self.start.y == self.end.y && self.start.x < self.end.x
            {
                (self.start, self.end)
            } else {
                (self.end, self.start)
            };

            let clip = |p: Point| Point {
                x: p.x.max(bounds.x).min(bounds.x + bounds.width),
                y: p.y.max(bounds.y).min(bounds.y + bounds.height),
            };

            if start.y < bounds.y {
                start = bounds.position();
            } else {
                start = clip(start);
            }

            if end.y > bounds.y + bounds.height {
                end = bounds.position() + Vector::from(bounds.size());
            } else {
                end = clip(end);
            }

            ((start.x - end.x).abs() > 1.0).then_some(Resolved { start, end })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Resolved {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

pub fn selection<P: text::Paragraph>(
    raw: Raw,
    bounds: Rectangle,
    paragraph: &P,
    value: &Value,
) -> Option<Selection> {
    let resolved = raw.resolve(bounds)?;

    let start_pos = relative(resolved.start, bounds);
    let end_pos = relative(resolved.end, bounds);

    let start = find_cursor_position(paragraph, value, start_pos)?;
    let end = find_cursor_position(paragraph, value, end_pos)?;

    (start != end).then(|| Selection {
        start: start.min(end),
        end: start.max(end),
    })
}

pub fn find_cursor_position<P: text::Paragraph>(
    paragraph: &P,
    value: &Value,
    cursor_position: Point,
) -> Option<usize> {
    let value = value.to_string();

    let char_offset =
        paragraph.hit_test(cursor_position).map(text::Hit::cursor)?;

    Some(
        unicode_segmentation::UnicodeSegmentation::graphemes(
            &value[..char_offset],
            true,
        )
        .count(),
    )
}

fn relative(point: Point, bounds: Rectangle) -> Point {
    point - Vector::new(bounds.x, bounds.y)
}
