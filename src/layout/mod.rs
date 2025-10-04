use skia_safe::{FontMgr, Point, Rect, Size};

use crate::{extensions::RectExt, graphics::Fonts};

pub mod text;

pub struct LayoutCtx {
    pub outer_main_rect: Rect,
    pub main_rect: Rect,
    pub side_rect: Rect,
    pub fonts: Fonts,
    pub image_size: Size,
}

impl LayoutCtx {
    pub fn new(image_size: Size) -> Self {
        let font_mgr = FontMgr::new();
        let mut style_set = font_mgr.match_family("Noto Sans");
        let fonts = Fonts::new(&mut style_set);

        let outer_main_rect = centered_rect(
            (image_size.width / 2.0, image_size.height - 300.0),
            (750.0, 200.0),
        )
        .moved_by(Point::new(-100.0, 0.0));

        let main_rect = outer_main_rect.with_bottom_offset(-50.0);
        let side_rect = Rect::from_xywh(
            outer_main_rect.right + 20.0,
            outer_main_rect.y(),
            200.0,
            200.0,
        );
        Self {
            outer_main_rect,
            main_rect,
            side_rect,
            fonts,
            image_size,
        }
    }
}

fn centered_rect(center: impl Into<Point>, size: impl Into<Size>) -> Rect {
    let center = center.into();
    let size = size.into();
    Rect::from_point_and_size(
        (center.x - size.width / 2.0, center.y - size.height / 2.0),
        size,
    )
}
