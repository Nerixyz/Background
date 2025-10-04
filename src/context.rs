use std::sync::{Arc, RwLock};

use skia_safe::{Color, Image, Paint, Point, RRect, Rect, Size, TextBlob};

use crate::{
    config::CONFIG,
    dwd::{Cache, Datapoint, icons::Msn},
    extensions::RectExt,
    graph::{
        self, HorizontalLine, PPrecipitationPlan, RadarPlan, RainPlan, SectionPlan, TemperaturePlan,
    },
    graphics::{mask_gradient_horiz, mask_gradient_vert},
    icons::IconRenderer,
    layout::{
        LayoutCtx,
        text::{Align, align_text},
    },
    paint::{
        BlurredSquircleItem, CurrentTime, ImageItem, LineItem, LinesItem, PaintLineItem, PathItem,
        Pipeline, RestoreOp, RrectItem, ShaderClipOp, TextItem, TextsItem,
    },
};

pub struct Context {
    pub cache: Arc<RwLock<Cache>>,
    pub plans: Option<Plans>,
    pub icons: IconRenderer<Msn>,
    pub bg_image: Image,

    pub layout_ctx: LayoutCtx,
}

pub struct Plans {
    pub sections: SectionPlan,
    pub temperature: Option<TemperaturePlan>,
    pub rain: Option<RainPlan>,
    pub p_precipitation: Option<PPrecipitationPlan>,
    pub horizontal_lines: Vec<HorizontalLine>,
    pub current: Option<Datapoint>,
    pub radar: Option<RadarPlan>,
}

impl Plans {
    pub fn new(cache: &Cache, ctx: &LayoutCtx) -> Self {
        let merged = Datapoint::merge_series_ref(&cache.report, &cache.forecast);
        let (overall, sections) = graph::plan_in(ctx.main_rect, &merged);
        let (temperature, mut horizontal_lines) = graph::create_temperature_path(&overall);
        let rain = graph::create_rain_plan(&overall, &mut horizontal_lines);
        let p_precipitation = graph::create_p_precipitation_plan(&overall);
        let current = cache
            .observation
            .clone()
            .or_else(|| merged.iter().filter(|x| x.is_report).next_back().cloned());
        let radar = graph::create_radar_plan(ctx.side_rect.with_inset((20.0, 0.0)), &cache.radar);

        Self {
            sections,
            temperature,
            rain,
            p_precipitation,
            horizontal_lines,
            current,
            radar,
        }
    }
}

impl Context {
    pub fn new(cache: Cache, bg_image: Image) -> Self {
        let layout_ctx =
            LayoutCtx::new(Size::new(bg_image.width() as f32, bg_image.height() as f32));
        Self {
            cache: Arc::new(RwLock::new(cache)),
            plans: None,
            icons: IconRenderer::new(),
            bg_image,
            layout_ctx,
        }
    }

    pub fn update(&mut self) -> bool {
        if Cache::refetch(&self.cache).unwrap() {
            self.replan();
            true
        } else {
            false
        }
    }

    pub fn replan(&mut self) {
        let cache = self.cache.read().unwrap();
        cache.to_file(CONFIG.cache_file()).unwrap();
        self.plans = Some(Plans::new(&cache, &self.layout_ctx));
    }

    pub fn relayout(&mut self, pipl: &mut Pipeline) {
        pipl.items.clear();
        pipl.add(ImageItem {
            image: self.bg_image.clone(),
            left_top: Point::new(0.0, 0.0),
        });

        self.main_pipeline(pipl);
        self.side_pipl(pipl);
    }

    pub fn main_pipeline(&mut self, pipl: &mut Pipeline) {
        let Some(ref plans) = self.plans else {
            return;
        };
        let outer_rect = self.layout_ctx.outer_main_rect;

        pipl.add(BlurredSquircleItem::new(outer_rect, 25.0, 30.0, 1.0));
        pipl.add(ShaderClipOp {
            shader: mask_gradient_vert(outer_rect),
            save: true,
        });
        pipl.add(LinesItem {
            points: plans
                .sections
                .sections
                .iter()
                .map(|s| {
                    (
                        Point::new(s.x, outer_rect.top),
                        Point::new(s.x, outer_rect.bottom),
                    )
                })
                .collect(),
            color: Color::from_argb(64, 255, 255, 255),
            stroke: 1.0,
        });
        pipl.add(CurrentTime {
            color: Color::from_argb(200, 255, 255, 255),
            stroke: 1.0,
            base_ts: plans.sections.near_base_ts,
            time_scale: plans.sections.near_minute_scale,
        });

        pipl.add(RestoreOp {}); // vert clip
        pipl.add(ShaderClipOp {
            shader: mask_gradient_horiz(outer_rect),
            save: true,
        });
        if let Some(ref rplan) = plans.rain {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_shader(rplan.gradient.clone());
            paint.set_color(Color::WHITE);
            paint.set_style(skia_safe::PaintStyle::Fill);
            pipl.add(PathItem {
                path: rplan.path.clone(),
                paint,
            });
        }
        if let Some(ref pplan) = plans.p_precipitation {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_shader(pplan.gradient.clone());
            paint.set_color(Color::WHITE);
            paint.set_style(skia_safe::PaintStyle::Stroke);
            pipl.add(PaintLineItem {
                points: pplan.points,
                paint,
            });
        }
        pipl.add(LinesItem {
            points: plans
                .horizontal_lines
                .iter()
                .map(|l| {
                    (
                        Point::new(outer_rect.left, l.y_pos - 0.5),
                        Point::new(outer_rect.right, l.y_pos - 0.5),
                    )
                })
                .collect(),
            color: Color::from_argb(64, 255, 255, 255),
            stroke: 1.0,
        });

        if let Some(ref temp) = plans.temperature {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_shader(temp.shader.clone());
            paint.set_stroke_width(1.5);
            paint.set_color(Color::WHITE);
            paint.set_style(skia_safe::PaintStyle::Stroke);
            pipl.add(PathItem {
                path: temp.path.clone(),
                paint,
            });
        }

        let mut texts = Vec::new();
        for s in &plans.sections.sections {
            let blob = TextBlob::new(&s.text, &self.layout_ctx.fonts.small).unwrap();
            texts.push((blob, Point::new(s.x + 3.0, outer_rect.bottom - 3.0)));
        }
        pipl.add(TextsItem {
            texts,
            color: Color::from_argb(160, 255, 255, 255),
        });

        pipl.add(RestoreOp {}); // horizontal clip

        let mut texts = Vec::new();
        for line in &plans.horizontal_lines {
            texts.push((
                TextBlob::new(
                    format!("{}°", line.temperature),
                    &self.layout_ctx.fonts.small,
                )
                .unwrap(),
                Point::new(outer_rect.left + 3.0, line.y_pos - 3.0),
            ));

            if let Some(r) = line.rain {
                let blob =
                    TextBlob::new(format!("{r:.1} mm"), &self.layout_ctx.fonts.small).unwrap();
                let w = blob.bounds().width();
                texts.push((
                    blob,
                    Point::new(outer_rect.right - 3.0 - w, line.y_pos - 3.0),
                ));
            }
        }

        pipl.add(TextsItem {
            texts,
            color: Color::from_argb(160, 255, 255, 255),
        });

        for it in plans.sections.sections.windows(2) {
            let cur = &it[0];
            let next = &it[1];
            let rect = Rect::from_xywh(
                (next.x + cur.x) / 2.0 - 12.0,
                outer_rect.bottom - 40.0,
                24.0,
                24.0,
            );
            if let Some(item) = self.icons.layout(&cur.data, rect) {
                pipl.add(item);
            }
        }

        pipl.add(RestoreOp {}); // squircle clip
    }

    pub fn side_pipl(&mut self, pipl: &mut Pipeline) {
        let Some(current) = self.plans.as_ref().and_then(|p| p.current.as_ref()) else {
            return;
        };
        pipl.add(BlurredSquircleItem::new(
            self.layout_ctx.side_rect,
            25.0,
            30.0,
            1.0,
        ));

        if let Some(temp) = current.temperature {
            let (blob, pos) = align_text(
                &format!("{temp:.*}°", if temp.fract() == 0.0 { 0 } else { 1 }),
                &self.layout_ctx.fonts.large,
                self.layout_ctx.side_rect.top_right() + Point::new(-20.0, 20.0),
                Align::TopRight,
            );
            pipl.add(TextItem {
                blob,
                pos,
                color: Color::WHITE,
            });
        }
        if let Some(svg) = self.icons.layout(
            current,
            Rect::from_xywh(
                self.layout_ctx.side_rect.left + 20.0,
                self.layout_ctx.side_rect.top + 20.0,
                40.0,
                40.0,
            ),
        ) {
            pipl.add(svg);
        }

        let mut y = self.layout_ctx.side_rect.top + 60.0;

        let mut texts = Vec::new();
        let label = |texts: &mut Vec<(TextBlob, Point)>, text: &str, y: f32| {
            texts.push(align_text(
                text,
                &self.layout_ctx.fonts.medium_light,
                (self.layout_ctx.side_rect.left + 20.0, y),
                Align::BottomLeft,
            ));
        };
        let unit = |texts: &mut Vec<(TextBlob, Point)>, text: &str, y: f32| {
            texts.push(align_text(
                text,
                &self.layout_ctx.fonts.medium,
                (self.layout_ctx.side_rect.right - 20.0, y),
                Align::BottomRight,
            ));
        };

        if let Some(cc) = current.cloud_cover {
            y += 25.0;
            label(&mut texts, "Cloud Cover", y);
            unit(&mut texts, &format!("{:.0}%", cc.round()), y);
        }
        if let Some(wind) = current.mean_wind {
            y += 25.0;
            label(&mut texts, "Wind Speed", y);
            unit(&mut texts, &format!("{wind:.1}\u{00A0}km/h"), y);
        }
        if let Some(h) = current.relative_humidity {
            y += 25.0;
            label(&mut texts, "Humidity", y);
            unit(
                &mut texts,
                &format!("{h:.*}%", if h.fract() == 0.0 { 0 } else { 1 }),
                y,
            );
        }

        if let Some(radar) = self.plans.as_ref().and_then(|p| p.radar.as_ref()) {
            const BOTTOM_OFF: f32 = 20.0;
            const HEIGHT: f32 = 10.0;
            let base_rect = Rect::from_ltrb(
                self.layout_ctx.side_rect.left + 20.0,
                self.layout_ctx.side_rect.bottom - BOTTOM_OFF - HEIGHT,
                self.layout_ctx.side_rect.right - 20.0,
                self.layout_ctx.side_rect.bottom - BOTTOM_OFF,
            );
            let rrect = RRect::new_rect_xy(base_rect, 5.0, 5.0);
            pipl.add(RrectItem {
                rect: rrect,
                shader: radar.shader.clone(),
                stroke: (Color::from_argb(150, 255, 255, 255), 0.5),
            });

            if let Some(ref ul) = radar.upper_label {
                pipl.add(LineItem {
                    points: (
                        Point::new(ul.x_pos, base_rect.top - 2.5),
                        Point::new(ul.x_pos, base_rect.bottom + 2.5),
                    ),
                    color: Color::WHITE,
                    stroke: 0.5,
                });
                texts.push(align_text(
                    &ul.text,
                    &self.layout_ctx.fonts.small,
                    (ul.x_pos, base_rect.top - 5.0),
                    Align::Bottom,
                ));
            }

            texts.push(align_text(
                &radar.start_text.text,
                &self.layout_ctx.fonts.small,
                (radar.start_text.x_pos, base_rect.bottom + 13.0),
                Align::BottomLeft,
            ));
            texts.push(align_text(
                &radar.end_text.text,
                &self.layout_ctx.fonts.small,
                (radar.end_text.x_pos, base_rect.bottom + 13.0),
                Align::BottomRight,
            ));
        }

        pipl.add(TextsItem {
            texts,
            color: Color::WHITE,
        });

        pipl.add(RestoreOp {}); // squircle clip
    }
}
