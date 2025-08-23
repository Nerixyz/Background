use skia_safe::{Canvas, Shader};

use crate::paint::{PaintCtx, Paintable};

pub struct ShaderClipOp {
    pub shader: Shader,
    pub save: bool,
}

impl Paintable for ShaderClipOp {
    fn paint(&mut self, canvas: &Canvas, _ctx: &mut PaintCtx<'_>) {
        if self.save {
            canvas.save();
        }
        canvas.clip_shader(self.shader.clone(), skia_safe::ClipOp::Intersect);
    }
}

pub struct RestoreOp {}

impl Paintable for RestoreOp {
    fn paint(&mut self, canvas: &Canvas, _ctx: &mut PaintCtx<'_>) {
        canvas.restore();
    }
}
