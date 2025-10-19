use skia_safe::{Color, Point, Rect, Shader, TileMode, gradient_shader, scalar};

use crate::extensions::RectExt;

pub fn mask_gradient_horiz(outer: Rect) -> Shader {
    let inner = outer.pad_by((20.0, 10.0).into());
    let pos_1 = (inner.left - outer.left) / (outer.right - outer.left);
    let pos_2 = (inner.right - outer.left) / (outer.right - outer.left);
    mask_gradient(((outer.left, 0.0), (outer.right, 0.0)), pos_1, pos_2)
}

pub fn mask_gradient_vert(outer: Rect) -> Shader {
    let inner = outer.pad_by((10.0, 10.0).into());
    let pos_1 = (inner.top - outer.top) / (outer.bottom - outer.top);
    let pos_2 = (inner.bottom - outer.top) / (outer.bottom - outer.top);
    mask_gradient(((0.0, outer.top), (0.0, outer.bottom)), pos_1, pos_2)
}

pub struct GradientBuilder {
    colors: Vec<Color>,
    positions: Vec<scalar>,
}

impl GradientBuilder {
    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn put(&mut self, color: Color, pos: scalar) {
        self.colors.push(color);
        self.positions.push(pos);
    }

    pub fn build(&self, points: (impl Into<Point>, impl Into<Point>)) -> Shader {
        gradient_shader::linear(
            points,
            &self.colors[..],
            &self.positions[..],
            TileMode::Clamp,
            None,
            None,
        )
        .unwrap()
    }
}

impl Default for GradientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AutoGradientBuilder {
    gb: GradientBuilder,
    point1: Point,
    point2: Point,
    div: f32,
    begin: f32,
}

impl AutoGradientBuilder {
    pub fn new_horizontal(points: (impl Into<Point>, impl Into<Point>)) -> Self {
        let point1 = points.0.into();
        let point2 = points.1.into();
        let div = point2.x - point1.x;
        let begin = point1.x;
        Self {
            gb: GradientBuilder::new(),
            point1,
            point2,
            div,
            begin,
        }
    }

    pub fn put(&mut self, color: Color, x: f32) {
        self.gb.put(color, (x - self.begin) / self.div);
    }

    pub fn build(&self) -> Shader {
        self.gb.build((self.point1, self.point2))
    }
}

fn mask_gradient(points: (impl Into<Point>, impl Into<Point>), pos1: f32, pos2: f32) -> Shader {
    gradient_shader::linear(
        points,
        &[
            Color::TRANSPARENT,
            Color::WHITE,
            Color::WHITE,
            Color::TRANSPARENT,
        ][..],
        &[0.0, pos1, pos2, 1.0][..],
        TileMode::Clamp,
        None,
        None,
    )
    .unwrap()
}
