use skia_safe::{Font, Point, TextBlob};

#[allow(unused)]
pub enum Align {
    TopLeft,
    TopCenter,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Center,
}

pub fn align_text(
    text: &str,
    font: &Font,
    pos: impl Into<Point>,
    align: Align,
) -> (TextBlob, Point) {
    let mut pos = pos.into();
    let blob = TextBlob::new(text, font).unwrap();
    let width = blob.bounds().width();
    let height = blob.bounds().height();
    match align {
        Align::TopLeft => {
            pos.y += height;
        }
        Align::TopCenter => {
            pos.x -= width / 2.0;
            pos.y += height;
        }
        Align::TopRight => {
            pos.x -= width;
            pos.y += height;
        }
        Align::Right => {
            pos.x -= width;
            pos.y += height / 2.0;
        }
        Align::BottomRight => {
            pos.x -= width;
        }
        Align::Bottom => {
            pos.x -= width / 2.0;
        }
        Align::BottomLeft => {}
        Align::Left => {
            pos.y += height / 2.0;
        }
        Align::Center => {
            pos.y += height / 2.0;
            pos.y -= width / 2.0;
        }
    }

    (blob, pos)
}
