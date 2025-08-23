use skia_safe::{Canvas, Paint};

use crate::layout::LayoutCtx;
use enum_dispatch::enum_dispatch;

mod image;
mod line;
mod ops;
mod path;
mod rect;
mod squircle;
mod svg;
mod text;

pub use image::ImageItem;
pub use line::{CurrentTime, LineItem, LinesItem, PaintLineItem};
pub use ops::{RestoreOp, ShaderClipOp};
pub use path::PathItem;
pub use rect::{RectItem, RrectItem};
pub use squircle::BlurredSquircleItem;
pub use svg::SvgItem;
pub use text::{TextItem, TextsItem};

pub struct PaintCtx<'a> {
    layout: &'a LayoutCtx, // not needed _yet_
    /// reusable paint for text - doesn't have a stroke width set
    text_paint: Paint,
    /// reusable paint for lines and squares
    paint: Paint,
}

impl<'a> PaintCtx<'a> {
    pub fn new(layout: &'a LayoutCtx) -> Self {
        let mut text_paint = Paint::default();
        text_paint.set_anti_alias(true);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        Self {
            layout,
            text_paint,
            paint,
        }
    }
}

#[enum_dispatch]
pub trait Paintable {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>);
}

#[enum_dispatch(Paintable)]
pub enum PaintableTy {
    BlurredSquircleItem,
    CurrentTime,
    ImageItem,
    LineItem,
    LinesItem,
    PaintLineItem,
    PathItem,
    RectItem,
    RestoreOp,
    RrectItem,
    ShaderClipOp,
    SvgItem,
    TextItem,
    TextsItem,
}

#[derive(Default)]
pub struct Pipeline {
    pub items: Vec<PaintableTy>,
}

impl Pipeline {
    pub fn add(&mut self, item: impl Into<PaintableTy>) {
        self.items.push(item.into());
    }

    pub fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        for it in &mut self.items {
            it.paint(canvas, ctx);
        }
    }
}
