use skia_safe::Color4f;

pub const fn luma(v: u8) -> Color4f {
    rgba4f(0, 0, 0, v)
}

pub const fn rgba4f(r: u8, g: u8, b: u8, a: u8) -> Color4f {
    const fn c(c: u8) -> f32 {
        (c as f32) * (1.0 / 255.0)
    }
    Color4f {
        r: c(r),
        g: c(g),
        b: c(b),
        a: c(a),
    }
}

pub const fn rgb4f(r: u8, g: u8, b: u8) -> Color4f {
    rgba4f(r, g, b, 255)
}
