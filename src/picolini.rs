use std::thread::JoinHandle;

use skia_safe::{Color, Path, Point, Rect, Shader, scalar};
use skia_util::{RectExt, gridify};
use weather_layout::{
    ColorMap,
    data::{YMapping, min_max_n_by},
    fmt,
    lines::create_interpolated_path,
};

use crate::config::CONFIG;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct DataItem {
    pub timestamp: i64,
    pub temperature: f32,
    pub iaq: f32,
    pub co2: f32,
}

impl Eq for DataItem {}

impl PartialEq for DataItem {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

pub struct PlanItem {
    pub path: Path,
    pub shader: Shader,
    pub text_point: Point,
    pub text: String,
}

pub struct PicoliniPlan {
    pub top_label: String,
    pub temperature: PlanItem,
    pub iaq: PlanItem,
    pub co2: PlanItem,
}

pub struct PicoliniCache {
    items: Vec<DataItem>,
}

pub struct RefreshHandle {
    hdl: JoinHandle<Vec<DataItem>>,
}

impl PicoliniCache {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn start_refresh() -> RefreshHandle {
        RefreshHandle {
            hdl: std::thread::spawn(get),
        }
    }

    pub fn collect_refresh(&mut self, hdl: RefreshHandle) -> bool {
        let Ok(res) = hdl.hdl.join() else {
            return false;
        };
        if res == self.items {
            return false;
        }
        self.items = res;
        true
    }

    pub fn plan(&self, rect: Rect) -> Option<PicoliniPlan> {
        plan(&self.items, rect)
    }
}

fn get() -> Vec<DataItem> {
    let Ok(mut res) = ureq::get(CONFIG.picolini_url())
        .header(
            "Authorization",
            format!("Bearer {}", CONFIG.access_secret()),
        )
        .call()
        .inspect_err(|e| {
            dbg!(e);
        })
    else {
        return Vec::new();
    };
    let Ok(mut json) = res.body_mut().read_json::<Vec<DataItem>>() else {
        return Vec::new();
    };
    sort_and_clear(&mut json);
    json
}

pub fn plan(items: &[DataItem], rect: Rect) -> Option<PicoliniPlan> {
    let current = items.last()?;
    let first = items.first().filter(|x| x.timestamp != current.timestamp)?;
    let [temp, iaq, co2] = gridify(rect, 10.0);

    let ts_span = (current.timestamp - first.timestamp) as f32;
    let graph_width = temp[0].width();
    let ts_to_x = |ts: i64| temp[0].left + ((ts - first.timestamp) as f32 / ts_span) * graph_width;

    let my_tz = jiff::tz::db().get("Europe/Berlin").unwrap();
    let start_zoned = jiff::Timestamp::from_millisecond(first.timestamp)
        .unwrap_or_default()
        .to_zoned(my_tz.clone());
    let cur_zoned = jiff::Timestamp::from_millisecond(current.timestamp)
        .unwrap_or_default()
        .to_zoned(my_tz);

    let top_label = format!(
        "{} — {}",
        start_zoned.strftime("%R"),
        cur_zoned.strftime("%R")
    );

    Some(PicoliniPlan {
        top_label,
        temperature: make_item::<TempColors>(
            temp,
            items,
            |it| it.temperature,
            ts_to_x,
            1.0,
            0.0,
            format!("{} °C", fmt::optional_fract(current.temperature)),
        ),
        iaq: make_item::<IaqColors>(
            iaq,
            items,
            |it| it.iaq,
            ts_to_x,
            50.0,
            0.0,
            format!("{:.0} IAQ", current.iaq),
        ),
        co2: make_item::<Co2Colors>(
            co2,
            items,
            |it| it.co2,
            ts_to_x,
            100.0,
            0.0,
            format!("{:.0} ppm", current.co2),
        ),
    })
}

fn make_item<M: ColorMap>(
    [graph_rect, text_rect]: [Rect; 2],
    items: &[DataItem],
    lookup: impl Fn(&DataItem) -> f32,
    ts_to_x: impl Fn(i64) -> f32,
    val_grid: f32,
    min_val: f32,
    text: String,
) -> PlanItem {
    let (path, shader) = layout_graph::<M>(graph_rect, items, lookup, ts_to_x, val_grid, min_val);
    PlanItem {
        path,
        shader,
        text_point: text_rect.center_right(),
        text,
    }
}

fn layout_graph<M: ColorMap>(
    rect: Rect,
    items: &[DataItem],
    lookup: impl Fn(&DataItem) -> f32,
    ts_to_x: impl Fn(i64) -> f32,
    val_grid: f32,
    min_val: f32,
) -> (Path, Shader) {
    let (min, max, _n) = min_max_n_by(items, |i| Some(lookup(i))).unwrap();
    let min = ((min / val_grid).round() * val_grid - val_grid).max(min_val);
    let max = (max / val_grid).round() * val_grid + val_grid;
    let mapping = YMapping::from_min_max(min, max, rect);
    let points = items
        .iter()
        .map(|it| Point::new(ts_to_x(it.timestamp), mapping.map(lookup(it))))
        .collect::<Vec<_>>();
    let path = create_interpolated_path(&points);

    let shader = skia_safe::gradient_shader::linear(
        ((0.0, mapping.map(M::MIN_T)), (0.0, mapping.map(M::MAX_T))),
        M::T_COLORS,
        M::T_POS,
        skia_safe::TileMode::Clamp,
        None,
        None,
    )
    .unwrap();

    (path, shader)
}

struct TempColors;
impl ColorMap for TempColors {
    const T_COLORS: &[Color] = &[
        Color::from_rgb(18, 230, 230),
        Color::from_rgb(85, 204, 0),
        Color::from_rgb(255, 247, 0),
        Color::from_rgb(255, 149, 0),
        Color::from_rgb(247, 15, 15),
    ];

    const T_POS: &[scalar] = &[
        Self::pos_of(15.0),
        Self::pos_of(18.0),
        Self::pos_of(20.0),
        Self::pos_of(22.0),
        Self::pos_of(23.0),
    ];

    const MIN_T: f32 = 15.0;

    const MAX_T: f32 = 23.0;

    fn map_rain(_value: f32) -> skia_safe::Color {
        Color::new(0)
    }
}
impl TempColors {
    const fn pos_of(v: scalar) -> scalar {
        (v - Self::MIN_T) / (Self::MAX_T - Self::MIN_T)
    }
}

struct IaqColors;
impl ColorMap for IaqColors {
    const T_COLORS: &[Color] = &[
        Color::from_rgb(0, 255, 0),
        Color::from_rgb(255, 255, 0),
        Color::from_rgb(255, 0, 0),
    ];

    const T_POS: &[scalar] = &[Self::pos_of(50.0), Self::pos_of(100.0), Self::pos_of(200.0)];

    const MIN_T: f32 = 0.0;

    const MAX_T: f32 = 200.0;

    fn map_rain(_value: f32) -> skia_safe::Color {
        Color::new(0)
    }
}
impl IaqColors {
    const fn pos_of(v: scalar) -> scalar {
        (v - Self::MIN_T) / (Self::MAX_T - Self::MIN_T)
    }
}

struct Co2Colors;
impl ColorMap for Co2Colors {
    const T_COLORS: &[Color] = &[
        Color::from_rgb(0, 255, 0),
        Color::from_rgb(255, 255, 0),
        Color::from_rgb(255, 0, 0),
    ];

    const T_POS: &[scalar] = &[
        Self::pos_of(400.0),
        Self::pos_of(1000.0),
        Self::pos_of(2000.0),
    ];

    const MIN_T: f32 = 0.0;

    const MAX_T: f32 = 2000.0;

    fn map_rain(_value: f32) -> skia_safe::Color {
        Color::new(0)
    }
}
impl Co2Colors {
    const fn pos_of(v: scalar) -> scalar {
        (v - Self::MIN_T) / (Self::MAX_T - Self::MIN_T)
    }
}

fn sort_and_clear(data: &mut Vec<DataItem>) {
    data.sort_by_key(|x| x.timestamp);
    let now = jiff::Timestamp::now().as_millisecond();
    let limit = now - (1000 * 60 * 60 * 5);
    data.retain(|x| x.timestamp >= limit);
}
