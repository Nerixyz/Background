use skia_safe::{Canvas, Color, Point, TextBlob};

use crate::paint::{PaintCtx, Paintable};

pub struct TextItem {
    pub blob: TextBlob,
    pub pos: Point,
    pub color: Color,
}

impl Paintable for TextItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx) {
        ctx.text_paint.set_color(self.color);
        canvas.draw_text_blob(&self.blob, self.pos, &ctx.text_paint);
    }
}

pub struct TextsItem {
    pub texts: Vec<(TextBlob, Point)>,
    pub color: Color,
}

impl Paintable for TextsItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        ctx.text_paint.set_color(self.color);
        for (blob, pos) in &self.texts {
            canvas.draw_text_blob(blob, *pos, &ctx.text_paint);
        }
    }
}
