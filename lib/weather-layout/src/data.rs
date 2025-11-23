use skia_safe::Rect;

pub fn min_max_n_by<T>(
    data: &[T],
    mut by: impl FnMut(&T) -> Option<f32>,
) -> Option<(f32, f32, usize)> {
    let mut minmax = None;
    let mut n_points = 0;
    for point in data {
        let Some(val) = by(point) else {
            continue;
        };
        match minmax {
            None => minmax = Some((val, val)),
            Some((min, max)) => minmax = Some((min.min(val), max.max(val))),
        }
        n_points += 1;
    }

    minmax.map(|(min, max)| (min, max, n_points))
}

pub struct YMapping {
    in_min: f32,
    in_range: f32,
    out_bottom: f32,
    out_height: f32,
}

impl YMapping {
    pub fn from_min_max(min: f32, max: f32, out: Rect) -> Self {
        Self {
            in_min: min,
            in_range: max - min,
            out_bottom: out.bottom,
            out_height: out.height(),
        }
    }

    pub fn map(&self, i: f32) -> f32 {
        let y_off = (i - self.in_min) * self.out_height / self.in_range;
        self.out_bottom - y_off
    }
}
