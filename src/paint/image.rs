use skia_safe::{Canvas, Image, Point};

use crate::paint::{PaintCtx, Paintable};

pub struct ImageItem {
    pub image: Image,
    pub left_top: Point,
}

impl Paintable for ImageItem {
    fn paint(&mut self, canvas: &Canvas, _ctx: &mut PaintCtx<'_>) {
        canvas.draw_image(&self.image, self.left_top, None);
    }
}
