use skia_safe::{Canvas, Color, Paint, PaintStyle, Point};

use crate::paint::{PaintCtx, Paintable};

pub struct LineItem {
    pub points: (Point, Point),
    pub color: Color,
    pub stroke: f32,
}

impl Paintable for LineItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        ctx.paint.set_style(PaintStyle::Stroke);
        ctx.paint.set_color(self.color);
        ctx.paint.set_stroke_width(self.stroke);
        canvas.draw_line(self.points.0, self.points.1, &ctx.paint);
    }
}

pub struct LinesItem {
    pub points: Vec<(Point, Point)>,
    pub color: Color,
    pub stroke: f32,
}

impl Paintable for LinesItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        ctx.paint.set_style(PaintStyle::Stroke);
        ctx.paint.set_color(self.color);
        ctx.paint.set_stroke_width(self.stroke);
        for (a, b) in &self.points {
            canvas.draw_line(*a, *b, &ctx.paint);
        }
    }
}

pub struct CurrentTime {
    pub color: Color,
    pub stroke: f32,
    pub base_ts: jiff::Timestamp,
    pub time_scale: f32,
}

impl Paintable for CurrentTime {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        let offset_x = (jiff::Timestamp::now() - self.base_ts)
            .total(jiff::Unit::Minute)
            .unwrap_or_default() as f32
            * self.time_scale;
        ctx.paint.set_style(PaintStyle::Stroke);
        ctx.paint.set_color(self.color);
        ctx.paint.set_stroke_width(self.stroke);
        let x = (ctx.layout.main_rect.x() + offset_x).round();
        canvas.draw_line(
            (x, ctx.layout.outer_main_rect.top),
            (x, ctx.layout.outer_main_rect.bottom),
            &ctx.paint,
        );
    }
}

pub struct PaintLineItem {
    pub points: (Point, Point),
    pub paint: Paint,
}

impl Paintable for PaintLineItem {
    fn paint(&mut self, canvas: &Canvas, _ctx: &mut PaintCtx<'_>) {
        canvas.draw_line(self.points.0, self.points.1, &self.paint);
    }
}
