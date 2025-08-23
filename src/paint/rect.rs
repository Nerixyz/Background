use skia_safe::{Canvas, Color, PaintStyle, RRect, Rect, Shader};

use crate::paint::{PaintCtx, Paintable};

pub struct RectItem {
    pub rect: Rect,
    pub color: Color,
}

impl Paintable for RectItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        ctx.paint.set_style(PaintStyle::Fill);
        ctx.paint.set_color(self.color);
        canvas.draw_rect(self.rect, &ctx.paint);
    }
}

pub struct RrectItem {
    pub rect: RRect,
    pub shader: Shader,
    pub stroke: (Color, f32),
}

impl Paintable for RrectItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        ctx.paint.set_shader(self.shader.clone());
        ctx.paint.set_color(Color::WHITE);
        ctx.paint.set_style(PaintStyle::Fill);
        canvas.draw_rrect(self.rect, &ctx.paint);
        ctx.paint.set_shader(None);
        ctx.paint.set_color(self.stroke.0);
        ctx.paint.set_stroke_width(self.stroke.1);
        ctx.paint.set_style(PaintStyle::Stroke);
        canvas.draw_rrect(self.rect, &ctx.paint);
    }
}
