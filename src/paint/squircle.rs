use skia_safe::{
    Canvas, Color, Color4f, Paint, PaintStyle, Path, Rect, TileMode,
    canvas::{SaveLayerFlags, SaveLayerRec},
    image_filters,
};

use crate::paint::{PaintCtx, Paintable};

pub struct BlurredSquircleItem {
    pub rect: Rect,
    pub path: Path,
    pub blur_paint: Paint,
    pub bg_color: Color4f,
    pub outline_color: Color,
}

impl Paintable for BlurredSquircleItem {
    fn paint(&mut self, canvas: &Canvas, ctx: &mut PaintCtx<'_>) {
        canvas.save();
        canvas.clip_path(&self.path, None, true);
        {
            let slr = SaveLayerRec::default()
                .bounds(&self.rect)
                .paint(&self.blur_paint)
                .flags(SaveLayerFlags::INIT_WITH_PREVIOUS);
            canvas.save_layer(&slr);
            canvas.draw_color(self.bg_color, None);
            canvas.restore(); // save layer
        }
        canvas.restore(); // clip path

        ctx.paint.set_style(PaintStyle::Stroke);
        ctx.paint.set_color(self.outline_color);
        ctx.paint.set_stroke_width(1.0);
        canvas.draw_path(&self.path, &ctx.paint);

        canvas.save();
        canvas.clip_path(&self.path, None, true);
    }
}

impl BlurredSquircleItem {
    pub fn new(rect: impl Into<Rect>, blur_radius: f32, radius: f32, smoothing: f32) -> Self {
        let rect = rect.into();
        let path = skia_util::squircle::create_path(rect, radius, smoothing);

        let filter =
            image_filters::blur((blur_radius, blur_radius), TileMode::Clamp, None, None).unwrap();
        let mut blur_paint = Paint::default();
        blur_paint.set_image_filter(filter);
        blur_paint.set_anti_alias(true);

        Self {
            rect,
            path,
            blur_paint,
            bg_color: Color4f::from_bytes_rgba(0x20ff_ffff),
            outline_color: Color::new(0x60ff_ffff),
        }
    }
}
