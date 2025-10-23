use skia_safe::{Point, Rect};
use skia_util::RectExt;

use crate::{EPD_HEIGHT, EPD_WIDTH};

pub struct LayoutCtx {
    pub top_rect: Rect,
    pub main_rect: Rect,
    pub bottom_left_rect: Rect,
    pub bottom_right_rect: Rect,

    pub mid_bottom_divider: (Point, Point),
    pub bottom_divider: (Point, Point),
}

impl Default for LayoutCtx {
    fn default() -> Self {
        let full = Rect::from_wh(EPD_WIDTH as f32, EPD_HEIGHT as f32);
        let bounded = full.with_inset((20, 20));
        let (top_rect, rem) = bounded.y_split_frac(0.1);
        let (main_rect, bottom_rect) = rem.with_top_offset(5.0).y_split_frac(0.7);
        let (bottom_left_rect, bottom_right_rect) =
            bottom_rect.with_top_offset(10.0).x_split_frac(0.5);

        let mid_bottom_y = bottom_rect.top + 5.0;
        let bottom_x = bottom_left_rect.right + 2.5;

        Self {
            top_rect,
            main_rect,
            bottom_left_rect: bottom_left_rect.with_right_offset(-5.0),
            bottom_right_rect: bottom_right_rect.with_left_offset(5.0),
            mid_bottom_divider: (
                Point::new(bottom_rect.left, mid_bottom_y),
                Point::new(bottom_rect.right, mid_bottom_y),
            ),
            bottom_divider: (
                Point::new(bottom_x, bottom_left_rect.top),
                Point::new(bottom_x, bottom_left_rect.bottom),
            ),
        }
    }
}
