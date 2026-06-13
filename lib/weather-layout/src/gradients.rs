use dwd_fetch::RadarReading;
use skia_safe::{
    Color4f, Shader, TileMode,
    gradient::{Colors, Gradient, Interpolation, shaders},
    scalar,
};
use skia_util::color::{luma, rgb4f, rgba4f};

pub trait ColorMap {
    const T_COLORS: &[Color4f];
    const T_POS: &[scalar];
    const MIN_T: f32;
    const MAX_T: f32;

    fn map_rain(value: f32) -> Color4f;
}

pub(crate) fn create_t_gradient<M: ColorMap>(bottom: f32, top: f32) -> Shader {
    shaders::linear_gradient(
        ((0.0, bottom), (0.0, top)),
        &Gradient::new(
            Colors::new(M::T_COLORS, Some(M::T_POS), TileMode::Clamp, None),
            Interpolation::default(),
        ),
        None,
    )
    .unwrap()
}

pub(crate) fn create_r_gradient<M: ColorMap>(
    values: &[RadarReading],
    x_range: (f32, f32),
    reading_to_x: impl Fn(&RadarReading) -> f32,
) -> Shader {
    let mut colors = Vec::with_capacity(values.len());
    let mut positions = Vec::with_capacity(values.len());

    for value in values {
        colors.push(M::map_rain(value.value));
        positions.push(reading_to_x(value));
    }

    shaders::linear_gradient(
        ((x_range.0, 0.0), (x_range.1, 0.0)),
        &Gradient::new(
            Colors::new(&colors[..], Some(&positions[..]), TileMode::Clamp, None),
            Interpolation::default(),
        ),
        None,
    )
    .unwrap()
}

fn interpolate_color(a: Color4f, b: Color4f, f: f32) -> Color4f {
    Color4f {
        r: a.r * (1.0 - f) + b.r * f,
        g: a.g * (1.0 - f) + b.g * f,
        b: a.b * (1.0 - f) + b.b * f,
        a: a.a * (1.0 - f) + b.a * f,
    }
}

pub struct Colorful;
pub struct Grayscale;

impl ColorMap for Colorful {
    const T_COLORS: &[Color4f] = t_colors::COLORS;
    const T_POS: &[scalar] = t_colors::POS;
    const MAX_T: f32 = t_colors::MAX_T;
    const MIN_T: f32 = t_colors::MIN_T;

    fn map_rain(value: f32) -> Color4f {
        r_colors::radar_color_for(value)
    }
}

impl ColorMap for Grayscale {
    const T_COLORS: &[Color4f] = &[luma(0), luma(0)];
    const T_POS: &[scalar] = &[0., 0.];
    const MAX_T: f32 = 1.0;
    const MIN_T: f32 = 0.0;

    fn map_rain(value: f32) -> Color4f {
        match value {
            0.0 => luma(0),
            ..=1.5 => interpolate_color(luma(0), luma(255), value / 1.5),
            _ => luma(255),
        }
    }
}

mod t_colors {
    use skia_safe::{Color4f, scalar};

    use super::rgb4f;

    // 40°
    const C_40: Color4f = rgb4f(247, 15, 92);
    const P_40: scalar = 1.0;
    // 30°
    const C_30: Color4f = rgb4f(247, 15, 15);
    const P_30: scalar = pos_of(30.0);
    // 20°
    const C_20: Color4f = rgb4f(255, 149, 0);
    const P_20: scalar = pos_of(20.0);
    // 10°
    const C_10: Color4f = rgb4f(255, 247, 0);
    const P_10: scalar = pos_of(10.0);
    // 5°
    const C_5: Color4f = rgb4f(85, 204, 0);
    const P_5: scalar = pos_of(5.0);
    // 0°
    const C_0: Color4f = rgb4f(18, 230, 230);
    const P_0: scalar = pos_of(0.0);
    // -5°
    const C_N5: Color4f = rgb4f(18, 138, 230);
    const P_N5: scalar = pos_of(-5.0);
    // -10°
    const C_N10: Color4f = rgb4f(128, 18, 230);
    const P_N10: scalar = pos_of(-10.0);
    // -20°
    const C_N20: Color4f = rgb4f(227, 14, 206);
    const P_N20: scalar = 0.0;

    pub const COLORS: &[skia_safe::Color4f] =
        &[C_N20, C_N10, C_N5, C_0, C_5, C_10, C_20, C_30, C_40];
    pub const POS: &[scalar] = &[P_N20, P_N10, P_N5, P_0, P_5, P_10, P_20, P_30, P_40];

    pub const MIN_T: f32 = -20.0;
    pub const MAX_T: f32 = 40.0;

    const fn pos_of(v: scalar) -> scalar {
        (v - MIN_T) / (MAX_T - MIN_T)
    }
}

mod r_colors {
    use skia_safe::Color4f;

    use crate::gradients::rgba4f;

    use super::{interpolate_color, rgb4f};

    // <= 0.5mm
    const C_0_5: Color4f = rgb4f(0x00, 0x92, 0x91);
    // <= 1.5mm
    const C_1_5: Color4f = rgb4f(0x40, 0xc7, 0x60);
    // <= 4.5mm
    const C_4_5: Color4f = rgb4f(0xdc, 0xd3, 0x18);
    // rest
    const C_REST: Color4f = rgb4f(0x9b, 0x0f, 0x6d);

    pub fn radar_color_for(v: f32) -> Color4f {
        match v {
            0.0 => rgba4f(0x28, 0x10, 0x9f, 0),
            ..=0.5 => interpolate_color(rgba4f(0x28, 0x10, 0x9f, 10), C_0_5, v * 2.0),
            ..=1.5 => interpolate_color(C_0_5, C_1_5, v - 0.5),
            ..=4.5 => interpolate_color(C_1_5, C_4_5, (v - 1.5) / 3.0),
            _ => C_REST,
        }
    }
}
