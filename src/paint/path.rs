use skia_safe::{Canvas, Paint, Path};

use crate::paint::{PaintCtx, Paintable};

pub struct PathItem {
    pub path: Path,
    pub paint: Paint,
}

impl Paintable for PathItem {
    fn paint(&mut self, canvas: &Canvas, _ctx: &mut PaintCtx<'_>) {
        canvas.draw_path(&self.path, &self.paint);
    }
}
