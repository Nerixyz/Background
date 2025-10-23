use skia_safe::Rect;

use dwd_fetch::{Cache, Datapoint};
use weather_layout::{
    self, Grayscale, HorizontalLine, PPrecipitationPlan, RadarPlan, RainPlan, SectionPlan,
    TemperaturePlan,
};

use crate::layout_ctx::LayoutCtx;

pub struct Plans {
    pub sections: SectionPlan,
    pub temperature: Option<TemperaturePlan>,
    pub rain: Option<RainPlan>,
    pub p_precipitation: Option<PPrecipitationPlan>,
    pub horizontal_lines: Vec<HorizontalLine>,
    pub current: Option<Datapoint>,
    pub radar: Option<RadarPlan>,
    pub inner_main_rect: Rect,
}

impl Plans {
    pub fn new(cache: &Cache, ctx: &LayoutCtx) -> Self {
        let points = Datapoint::merge_series_ref(&cache.report, &cache.forecast);
        let inner_rect = ctx.main_rect.with_inset((20.0, 20.0));
        let (plan, sections) = weather_layout::plan_in(inner_rect, &points);
        let (temperature, mut horizontal_lines) =
            weather_layout::create_temperature_path::<Grayscale>(&plan);
        let rain = weather_layout::create_rain_plan::<Grayscale>(&plan, &mut horizontal_lines);
        let p_precipitation = weather_layout::create_p_precipitation_plan(&plan);

        let current = cache
            .observation
            .clone()
            .or_else(|| points.iter().rfind(|x| x.is_report).cloned());
        let radar = weather_layout::create_radar_plan::<Grayscale>(
            ctx.bottom_left_rect.with_inset((20.0, 0.0)),
            &cache.radar,
        );

        Self {
            sections,
            temperature,
            rain,
            p_precipitation,
            horizontal_lines,
            current,
            radar,
            inner_main_rect: inner_rect,
        }
    }
}
