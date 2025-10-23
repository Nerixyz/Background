use skia_safe::{Point, Rect, Size};

pub trait RectExt {
    fn pad_by(&self, pad: Size) -> Self;
    fn with_left_offset(&self, offset: f32) -> Self;
    fn with_top_offset(&self, offset: f32) -> Self;
    fn with_right_offset(&self, offset: f32) -> Self;
    fn with_bottom_offset(&self, offset: f32) -> Self;

    fn top_left(&self) -> Point;
    fn bottom_left(&self) -> Point;
    fn top_right(&self) -> Point;
    fn bottom_right(&self) -> Point;

    fn bottom_center(&self) -> Point;

    fn y_split_frac(&self, f: f32) -> (Rect, Rect);
    fn x_split_frac(&self, f: f32) -> (Rect, Rect);

    fn moved_by(&self, p: Point) -> Self;
}

impl RectExt for Rect {
    fn pad_by(&self, pad: Size) -> Self {
        Self::from_ltrb(
            self.left + pad.width,
            self.top + pad.height,
            self.right - pad.width,
            self.bottom - pad.height,
        )
    }

    fn with_left_offset(&self, offset: f32) -> Self {
        Self::from_ltrb(self.left + offset, self.top, self.right, self.bottom)
    }

    fn with_top_offset(&self, offset: f32) -> Self {
        Self::from_ltrb(self.left, self.top + offset, self.right, self.bottom)
    }

    fn with_right_offset(&self, offset: f32) -> Self {
        Self::from_ltrb(self.left, self.top, self.right + offset, self.bottom)
    }

    fn with_bottom_offset(&self, offset: f32) -> Self {
        Self::from_ltrb(self.left, self.top, self.right, self.bottom + offset)
    }

    fn top_left(&self) -> Point {
        Point {
            x: self.left,
            y: self.top,
        }
    }

    fn bottom_left(&self) -> Point {
        Point {
            x: self.left,
            y: self.bottom,
        }
    }

    fn top_right(&self) -> Point {
        Point {
            x: self.right,
            y: self.top,
        }
    }

    fn bottom_right(&self) -> Point {
        Point {
            x: self.right,
            y: self.bottom,
        }
    }

    fn bottom_center(&self) -> Point {
        Point {
            x: self.center_x(),
            y: self.bottom,
        }
    }

    fn moved_by(&self, p: Point) -> Self {
        Self {
            left: self.left + p.x,
            top: self.top + p.y,
            right: self.right + p.x,
            bottom: self.bottom + p.y,
        }
    }

    fn x_split_frac(&self, f: f32) -> (Rect, Rect) {
        let split = self.left + self.width() * f;
        (
            Rect::from_ltrb(self.left, self.top, split, self.bottom),
            Rect::from_ltrb(split, self.top, self.right, self.bottom),
        )
    }

    fn y_split_frac(&self, f: f32) -> (Rect, Rect) {
        let split = self.top + self.height() * f;
        (
            Rect::from_ltrb(self.left, self.top, self.right, split),
            Rect::from_ltrb(self.left, split, self.right, self.bottom),
        )
    }
}

pub trait PointExt {
    fn moved_y(&self, off: f32) -> Self;
    fn moved_x(&self, off: f32) -> Self;

    fn moved(&self, x_off: f32, y_off: f32) -> Self;
}

impl PointExt for Point {
    fn moved_y(&self, off: f32) -> Self {
        Self {
            x: self.x,
            y: self.y + off,
        }
    }

    fn moved_x(&self, off: f32) -> Self {
        Self {
            x: self.x + off,
            y: self.y,
        }
    }

    fn moved(&self, x_off: f32, y_off: f32) -> Self {
        Self {
            x: self.x + x_off,
            y: self.y + y_off,
        }
    }
}
