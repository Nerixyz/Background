use jiff::ToSpan;
use skia_safe::{Color, Path, Point, Rect, Shader};

use crate::{
    dwd::{Datapoint, RadarReading},
    extensions::RectExt,
    graphics::AutoGradientBuilder,
};

struct GraphPoint<'a> {
    pub x_pos: f32,
    pub data: &'a Datapoint,
}

#[derive(Debug)]
pub struct Section {
    pub x: f32,
    pub text: String,
    pub data: Datapoint,
}

pub struct SectionPlan {
    pub sections: Vec<Section>,

    pub near_base_ts: jiff::Timestamp,
    pub near_minute_scale: f32,
}

pub struct Plan<'a> {
    points: Vec<GraphPoint<'a>>,
    rect: Rect,
}

const BEGIN_HOURS: i32 = 8;
const NEAR_FUT_HOURS: i32 = 12;
const END_HOURS: i32 = 24 * 4; // yes, not every day has 24h, i know...

const BEGIN_MINUTES: i32 = BEGIN_HOURS * 60;
const NEAR_FUT_MINUTES: i32 = NEAR_FUT_HOURS * 60;
const END_MINUTES: i32 = END_HOURS * 60;

const FIRST_SECTION_MINUTES: i32 = BEGIN_MINUTES + NEAR_FUT_MINUTES;

pub fn plan_in<'a>(rect: skia_safe::Rect, all_points: &'a [Datapoint]) -> (Plan<'a>, SectionPlan) {
    let now = jiff::Timestamp::now()
        .round(
            jiff::TimestampRound::new()
                .smallest(jiff::Unit::Hour)
                .mode(jiff::RoundMode::Trunc),
        )
        .unwrap();
    let begin = now - BEGIN_HOURS.hours();
    let near_fut_ts = now + NEAR_FUT_HOURS.hours();
    let end = now + END_HOURS.hours();

    let begin_i = all_points.partition_point(|it| it.timestamp < begin);
    let future_i = all_points.partition_point(|it| it.timestamp < near_fut_ts);
    let end_i = all_points.partition_point(|it| it.timestamp < end);

    let near = &all_points[begin_i..future_i];
    let far_fut = &all_points[future_i..end_i];

    let mut plan = Vec::with_capacity(near.len() + far_fut.len());
    let mut sections = Vec::new();
    let section_size = rect.width() / 3.0;
    calc_x_offsets(
        near,
        rect.left,
        2.0 * section_size,
        FIRST_SECTION_MINUTES as f32,
        begin,
        &mut plan,
    );

    let my_tz = jiff::tz::db().get("Europe/Berlin").unwrap();

    for (_, p) in plan
        .iter()
        .enumerate()
        .rev()
        .step_by(2)
        .filter(|it| it.0 != 0)
    {
        let zoned = p.data.timestamp.to_zoned(my_tz.clone());
        sections.push(Section {
            x: p.x_pos,
            text: zoned.strftime("%H").to_string(),
            data: p.data.clone(),
        });
    }

    let first_day = plan.first().map(|p| p.data.local_ts.day()).unwrap_or(42);
    let mut cur_day = plan.last().map(|p| p.data.local_ts.day()).unwrap_or(42);
    let mut fut_start = plan.len();

    // The last section of the near future will be merged with the far future and show as a day
    if cur_day == first_day + 1 {
        sections.sort_by_key(|s| s.data.timestamp);
        sections.pop();

        cur_day -= 1;
        fut_start -= 1;
    }

    calc_x_offsets(
        far_fut,
        rect.left + 2.0 * section_size,
        section_size,
        (END_MINUTES - NEAR_FUT_MINUTES) as f32,
        near_fut_ts,
        &mut plan,
    );

    let mut pending_sec = None;
    for p in &plan[fut_start..plan.len() - 3] {
        let ts = p.data.timestamp.to_zoned(my_tz.clone());
        if ts.day() != cur_day {
            if let Some(s) = pending_sec.take() {
                sections.push(s);
            }
            pending_sec = Some(Section {
                x: p.x_pos,
                text: ts.strftime("%a").to_string(),
                data: p.data.clone(),
            });
            cur_day = ts.day();
            continue;
        }
        if let Some(ref mut s) = pending_sec
            && ts.hour() >= 14
            && !p.data.condition.is_none()
        {
            s.data = p.data.clone();
            sections.push(pending_sec.take().unwrap());
        }
    }
    if let Some(s) = pending_sec.take() {
        sections.push(s);
    }
    sections.sort_by_key(|s| s.data.timestamp);

    (
        Plan { points: plan, rect },
        SectionPlan {
            sections,
            near_base_ts: begin,
            near_minute_scale: (2.0 * section_size) / (FIRST_SECTION_MINUTES as f32),
        },
    )
}

fn calc_x_offsets<'a>(
    src: &'a [Datapoint],
    base_off: f32,
    scale: f32,
    total_minutes: f32,
    base_ts: jiff::Timestamp,
    dst: &mut Vec<GraphPoint<'a>>,
) {
    for p in src {
        let offset = (p.timestamp - base_ts).total(jiff::Unit::Minute).unwrap() as f32 * scale
            / total_minutes;
        dst.push(GraphPoint {
            x_pos: base_off + offset,
            data: p,
        });
    }
}

pub struct HorizontalLine {
    pub y_pos: f32,
    pub temperature: i32, // rounded
    pub rain: Option<f32>,
}

pub struct TemperaturePlan {
    pub path: Path,
    pub shader: Shader,
}

pub fn create_temperature_path(plan: &Plan) -> (Option<TemperaturePlan>, Vec<HorizontalLine>) {
    // first, determine the bounds
    let mut minmax = None;
    let mut n_points = 0;
    for point in &plan.points {
        let Some(val) = point.data.temperature else {
            continue;
        };
        match minmax {
            None => minmax = Some((val, val)),
            Some((min, max)) => minmax = Some((min.min(val), max.max(val))),
        }
        n_points += 1;
    }
    let Some((mut min, mut max)) = minmax else {
        return (None, Vec::new());
    };
    min = (min / 5.0).round() * 5.0 - 5.0;
    max = (max / 5.0).round() * 5.0 + 5.0;
    let range = max - min;
    let height = plan.rect.height();

    let mut points = Vec::with_capacity(n_points);
    for p in &plan.points {
        let Some(t) = p.data.temperature else {
            continue;
        };
        let y_off = (t - min) * height / range;
        points.push(Point::new(p.x_pos, plan.rect.bottom - y_off));
    }

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

    let temp_to_y_scale = plan.rect.height() / (max - min).max(1.0);
    let temp_to_y = |temp: f32| plan.rect.bottom + (min - temp) * temp_to_y_scale;

    let shader = create_t_gradient(temp_to_y(t_colors::MIN_T), temp_to_y(t_colors::MAX_T));
    let mut horizontal = Vec::new();
    {
        let mut t = min + 5.0;
        while t < max {
            horizontal.push(HorizontalLine::from_temp(t, temp_to_y));
            t += 5.0;
        }
    }

    (Some(TemperaturePlan { path, shader }), horizontal)
}

impl HorizontalLine {
    pub fn from_temp(temperature: f32, temp_to_y: impl Fn(f32) -> f32) -> Self {
        Self {
            y_pos: temp_to_y(temperature).round(),
            temperature: temperature as i32,
            rain: None,
        }
    }

    pub fn update_rain(&mut self, rect: Rect, max_rain: f32) {
        self.rain = Some((rect.bottom - self.y_pos) * max_rain / rect.height())
    }
}

pub struct RainPlan {
    pub path: Path,
    pub gradient: Shader,
}

pub fn create_rain_plan(plan: &Plan, horizontal_lines: &mut [HorizontalLine]) -> Option<RainPlan> {
    let max_value = plan
        .points
        .iter()
        .filter_map(|it| it.data.precipitation)
        .fold(0.0, f32::max);
    if max_value == 0.0 {
        return None;
    };
    let max_value = max_value.max(3.0) + 0.5;

    horizontal_lines
        .iter_mut()
        .for_each(|l| l.update_rain(plan.rect, max_value));
    let area_height = plan.rect.height();

    let mut pending_point = Point::new(plan.rect.left, plan.rect.bottom);
    let mut path = Path::new();
    let mut gradient =
        AutoGradientBuilder::new_horizontal((plan.rect.top_left(), plan.rect.top_right()));
    path.move_to(pending_point);
    for it in plan.points.windows(2) {
        let cur = &it[0];
        let next = &it[1];
        let value = cur.data.precipitation.unwrap_or_default();
        let width = (next.x_pos - cur.x_pos) / 2.0;
        let cur_point = Point::new(
            cur.x_pos + width,
            plan.rect.bottom - (value * area_height / max_value),
        );
        path.cubic_to(pending_point, (cur.x_pos, cur_point.y), cur_point);
        gradient.put(r_colors::radar_color_for(value), cur_point.x);
        pending_point = Point::new(next.x_pos, cur_point.y);
    }
    let end = Point::new(pending_point.x, plan.rect.bottom);
    path.cubic_to(pending_point, end, end);
    path.close();
    let gradient = gradient.build();

    Some(RainPlan { path, gradient })
}

// % chance for precipitation
pub struct PPrecipitationPlan {
    pub points: (Point, Point),
    pub gradient: Shader,
}

pub fn create_p_precipitation_plan(plan: &Plan) -> Option<PPrecipitationPlan> {
    let no_rain = plan
        .points
        .iter()
        .filter_map(|it| it.data.p_precipitation)
        .all(|it| it == 0.0);
    if no_rain {
        return None;
    };

    let mut gradient =
        AutoGradientBuilder::new_horizontal((plan.rect.top_left(), plan.rect.top_right()));
    for it in plan.points.windows(2) {
        let cur = &it[0];
        let next = &it[1];
        let value = cur.data.p_precipitation.unwrap_or_default();
        let width = (next.x_pos - cur.x_pos) / 2.0;
        let cur_point = Point::new(cur.x_pos + width, 0.0);
        let color = Color::from_argb((value * 2.5) as u8, 255, 255, 255);
        gradient.put(color, cur_point.x);
    }
    let gradient = gradient.build();

    Some(PPrecipitationPlan {
        points: (plan.rect.bottom_left(), plan.rect.bottom_right()),
        gradient,
    })
}

#[derive(Debug)]
pub struct XLabel {
    pub x_pos: f32,
    pub text: String, // TODO: use a TextBlob
}

#[derive(Debug)]
pub struct RadarPlan {
    pub shader: Shader,
    pub upper_label: Option<XLabel>,
    pub start_text: XLabel,
    pub end_text: XLabel,
}

pub fn create_radar_plan(inner_rect: Rect, values: &[RadarReading]) -> Option<RadarPlan> {
    if values.len() < 2 || values.iter().all(|v| v.value == 0.0) {
        return None;
    }
    let first = values.first().unwrap();
    let last = values.last().unwrap();

    let start_s = first.timestamp.as_second();
    let duration = (last.timestamp.as_second() - start_s) as f32;
    let start_s = start_s as f32;

    let reading_to_x = |r: &RadarReading| (r.timestamp.as_second() as f32 - start_s) / duration;
    let reading_to_lbl = |r: &RadarReading| XLabel {
        x_pos: inner_rect.left + reading_to_x(r) * inner_rect.width(),
        text: r.local_ts.strftime("%H:%M").to_string(),
    };

    let shader = create_r_gradient(values, (inner_rect.left, inner_rect.right), reading_to_x);
    let is_raining = values.first().unwrap().value != 0.0;
    let upper_label = if is_raining {
        values.iter().find(|x| x.value == 0.0).map(reading_to_lbl)
    } else {
        values.iter().find(|x| x.value != 0.0).map(reading_to_lbl)
    };
    let start_text = reading_to_lbl(first);
    let end_text = reading_to_lbl(last);

    Some(RadarPlan {
        shader,
        upper_label,
        start_text,
        end_text,
    })
}

mod t_colors {
    use skia_safe::{Color, scalar};

    // 40°
    const C_40: Color = Color::from_rgb(247, 15, 92);
    const P_40: scalar = 1.0;
    // 30°
    const C_30: Color = Color::from_rgb(247, 15, 15);
    const P_30: scalar = pos_of(30.0);
    // 20°
    const C_20: Color = Color::from_rgb(255, 149, 0);
    const P_20: scalar = pos_of(20.0);
    // 10°
    const C_10: Color = Color::from_rgb(255, 247, 0);
    const P_10: scalar = pos_of(10.0);
    // 5°
    const C_5: Color = Color::from_rgb(85, 204, 0);
    const P_5: scalar = pos_of(5.0);
    // 0°
    const C_0: Color = Color::from_rgb(18, 230, 230);
    const P_0: scalar = pos_of(0.0);
    // -5°
    const C_N5: Color = Color::from_rgb(18, 138, 230);
    const P_N5: scalar = pos_of(-5.0);
    // -10°
    const C_N10: Color = Color::from_rgb(128, 18, 230);
    const P_N10: scalar = pos_of(-10.0);
    // -20°
    const C_N20: Color = Color::from_rgb(227, 14, 206);
    const P_N20: scalar = 0.0;

    pub const COLORS: &[skia_safe::Color] = &[C_N20, C_N10, C_N5, C_0, C_5, C_10, C_20, C_30, C_40];
    pub const POS: &[scalar] = &[P_N20, P_N10, P_N5, P_0, P_5, P_10, P_20, P_30, P_40];

    pub const MIN_T: f32 = -20.0;
    pub const MAX_T: f32 = 40.0;

    const fn pos_of(v: scalar) -> scalar {
        (v - MIN_T) / (MAX_T - MIN_T)
    }
}

fn create_t_gradient(bottom: f32, top: f32) -> Shader {
    skia_safe::gradient_shader::linear(
        ((0.0, bottom), (0.0, top)),
        t_colors::COLORS,
        t_colors::POS,
        skia_safe::TileMode::Clamp,
        None,
        None,
    )
    .unwrap()
}

mod r_colors {
    use skia_safe::Color;

    use crate::graph::interpolate_color;

    // <= 0.5mm
    const C_0_5: Color = Color::from_rgb(0x00, 0x92, 0x91);
    // <= 1.5mm
    const C_1_5: Color = Color::from_rgb(0x40, 0xc7, 0x60);
    // <= 4.5mm
    const C_4_5: Color = Color::from_rgb(0xdc, 0xd3, 0x18);
    // rest
    const C_REST: Color = Color::from_rgb(0x9b, 0x0f, 0x6d);

    pub fn radar_color_for(v: f32) -> Color {
        match v {
            0.0 => Color::from_argb(0, 0x28, 0x10, 0x9f),
            ..=0.5 => interpolate_color(Color::from_argb(10, 0x28, 0x10, 0x9f), C_0_5, v * 2.0),
            ..=1.5 => interpolate_color(C_0_5, C_1_5, v - 0.5),
            ..=4.5 => interpolate_color(C_1_5, C_4_5, (v - 1.5) / 3.0),
            _ => C_REST,
        }
    }
}

fn create_r_gradient(
    values: &[RadarReading],
    x_range: (f32, f32),
    reading_to_x: impl Fn(&RadarReading) -> f32,
) -> Shader {
    let mut colors = Vec::with_capacity(values.len());
    let mut positions = Vec::with_capacity(values.len());

    for value in values {
        colors.push(r_colors::radar_color_for(value.value));
        positions.push(reading_to_x(value));
    }
    skia_safe::gradient_shader::linear(
        ((x_range.0, 0.0), (x_range.1, 0.0)),
        &colors[..],
        &positions[..],
        skia_safe::TileMode::Clamp,
        None,
        None,
    )
    .unwrap()
}

fn interpolate_color(a: Color, b: Color, f: f32) -> Color {
    Color::from_argb(
        ((a.a() as f32 * (1.0 - f)) + b.a() as f32 * f) as u8,
        ((a.r() as f32 * (1.0 - f)) + b.r() as f32 * f) as u8,
        ((a.g() as f32 * (1.0 - f)) + b.g() as f32 * f) as u8,
        ((a.b() as f32 * (1.0 - f)) + b.b() as f32 * f) as u8,
    )
}
