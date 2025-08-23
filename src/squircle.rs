use skia_safe::{Path, PathDirection, Rect, Size, path::ArcSize};

#[derive(Debug)]
struct CornerParams {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    p: f32,
    circular_section_length: f32,
}

impl CornerParams {
    fn new(size: Size, radius: f32, smoothing: f32) -> Self {
        let max_radius = size.width.min(size.height) / 2.0;
        let radius = radius.min(max_radius);
        let p = ((1.0 + smoothing) * radius).min(max_radius);

        let (angle_alpha, angle_beta) = if radius <= max_radius / 2.0 {
            (45.0 * smoothing, 90.0 * (1.0 - smoothing))
        } else {
            let diff_ratio = (radius - max_radius / 2.0) / (max_radius / 2.0);
            (
                45.0 * smoothing * (1.0 - diff_ratio),
                90.0 * (1.0 - smoothing * (1.0 - diff_ratio)),
            )
        };

        let angle_theta = (90.0 - angle_beta) / 2.0;
        let p3_to_p4_distance = radius * (angle_theta / 2.0).to_radians().tan();
        let circular_section_length =
            (angle_beta / 2.0).to_radians().sin() * radius * 2.0f32.sqrt();

        let c = p3_to_p4_distance * angle_alpha.to_radians().cos();
        let d = c * angle_alpha.to_radians().tan();
        let b = (p - circular_section_length - c - d) / 3.0;
        let a = 2.0 * b;

        Self {
            a,
            b,
            c,
            d,
            p,
            circular_section_length,
        }
    }
}

// From https://github.com/yjb94/react-native-squircle-skia
pub fn create_path(rect: impl Into<Rect>, radius: f32, smoothing: f32) -> Path {
    let rect = rect.into();
    let size = rect.size();

    let mut path = Path::new();
    let CornerParams {
        a,
        b,
        c,
        d,
        p,
        circular_section_length,
    } = CornerParams::new(size, radius, smoothing);
    let x = rect.x();
    let y = rect.y();
    let Size { width, height } = size;

    // top right
    path.move_to((x + (width / 2.0).max(width - p), y));
    path.cubic_to(
        (x + width - (p - a), y),
        (x + width - (p - a - b), y),
        (x + width - (p - a - b - c), y + d),
    );
    path.r_arc_to_rotated(
        (radius, radius),
        0.0,
        ArcSize::Small,
        PathDirection::CW,
        (circular_section_length, circular_section_length),
    );
    path.cubic_to(
        (x + width, y + p - a - b),
        (x + width, y + p - a),
        (x + width, y + (height / 2.0).min(p)),
    );

    // bottom right
    path.line_to((x + width, y + (height / 2.0).max(height - p)));
    path.cubic_to(
        (x + width, y + height - (p - a)),
        (x + width, y + height - (p - a - b)),
        (x + width - d, y + height - (p - a - b - c)),
    );
    path.r_arc_to_rotated(
        (radius, radius),
        0.0,
        ArcSize::Small,
        PathDirection::CW,
        (-circular_section_length, circular_section_length),
    );
    path.cubic_to(
        (x + width - (p - a - b), y + height),
        (x + width - (p - a), y + height),
        (x + (width / 2.0).max(width - p), y + height),
    );

    // bottom left
    path.line_to((x + (width / 2.0).min(p), y + height));
    path.cubic_to(
        (x + p - a, y + height),
        (x + p - a - b, y + height),
        (x + p - a - b - c, y + height - d),
    );
    path.r_arc_to_rotated(
        (radius, radius),
        0.0,
        ArcSize::Small,
        PathDirection::CW,
        (-circular_section_length, -circular_section_length),
    );
    path.cubic_to(
        (x, y + height - (p - a - b)),
        (x, y + height - (p - a)),
        (x, y + (height / 2.0).max(height - p)),
    );

    // top left
    path.line_to((x, y + (height / 2.0).min(p)));
    path.cubic_to(
        (x, y + p - a),
        (x, y + p - a - b),
        (x + d, y + p - a - b - c),
    );
    path.r_arc_to_rotated(
        (radius, radius),
        0.0,
        ArcSize::Small,
        PathDirection::CW,
        (circular_section_length, -circular_section_length),
    );
    path.cubic_to(
        (x + p - a - b, y),
        (x + p - a, y),
        (x + (width / 2.0).min(p), y),
    );
    path.close();

    path
}
