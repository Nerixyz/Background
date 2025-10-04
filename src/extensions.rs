use skia_safe::{Point, Rect, Size};

pub trait OptionExt<T> {
    fn or_assign(&mut self, other: Option<T>);
}

impl<T> OptionExt<T> for Option<T> {
    fn or_assign(&mut self, other: Option<T>) {
        if self.is_none() {
            *self = other;
        }
    }
}

#[allow(unused)]
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

    fn moved_by(&self, p: Point) -> Self {
        Self {
            left: self.left + p.x,
            top: self.top + p.y,
            right: self.right + p.x,
            bottom: self.bottom + p.y,
        }
    }
}

pub trait WinRectExt {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}

impl WinRectExt for windows::Win32::Foundation::RECT {
    fn width(&self) -> i32 {
        self.right - self.left
    }

    fn height(&self) -> i32 {
        self.bottom - self.top
    }
}
