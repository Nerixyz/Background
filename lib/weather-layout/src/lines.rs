use skia_safe::{Path, Point};

pub fn create_interpolated_path(points: &[Point]) -> Path {
    let mut path = Path::new();
    path.move_to(points[0]);

    let mut pending_point = points[0];
    for it in points.windows(3) {
        let prev = it[0];
        let cur = it[1];
        let next = it[2];

        let x_dist = next.x - prev.x;
        let handle_range = x_dist / 8.0;
        let y_off = (next.y - prev.y) * handle_range / x_dist;
        let grad = Point::new(handle_range, y_off);
        path.cubic_to(pending_point, cur - grad, cur);
        pending_point = cur + grad;
    }

    if points.len() >= 2 {
        let cur = points[points.len() - 1];
        path.cubic_to(pending_point, cur, cur);
    }
    path
}
