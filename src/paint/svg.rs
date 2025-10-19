use crate::paint::{PaintCtx, Paintable};
use skia_safe::{Canvas, Rect, svg::Dom};
use skia_util::RectExt;

pub struct SvgItem {
    pub dom: Dom,
    pub rect: Rect,
}

impl Paintable for SvgItem {
    fn paint(&mut self, canvas: &Canvas, _ctx: &mut PaintCtx<'_>) {
        self.dom.set_container_size(self.rect.size());
        canvas.translate(self.rect.top_left());
        self.dom.render(canvas);
        canvas.reset_matrix();
    }
}
