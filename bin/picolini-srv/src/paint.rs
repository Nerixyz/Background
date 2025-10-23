use icu_datetime::{fieldsets::YMDE, input::Date};
use icu_locale::locale;
use skia_safe::{Canvas, Color, Paint, PaintStyle, Point, RRect, Rect};
use skia_util::{
    PointExt, RectExt,
    text::{Align, align_text},
};

use crate::{fonts::Fonts, layout_ctx::LayoutCtx, plan::Plans};

pub struct PaintCtx {
    pub fonts: Fonts,
}

pub fn main(canvas: &Canvas, plans: &Plans, layout_ctx: &LayoutCtx, paint_ctx: &PaintCtx) {
    let mut p = Paint::default();

    p.set_color(Color::from_rgb(0, 0, 0));
    p.set_stroke(true);
    p.set_stroke_width(1.0);
    let now_x = plans.inner_main_rect.left
        + (jiff::Timestamp::now() - plans.sections.near_base_ts)
            .total(jiff::Unit::Minute)
            .unwrap_or_default() as f32
            * plans.sections.near_minute_scale;
    canvas.draw_line(
        (now_x, plans.inner_main_rect.top),
        (now_x, plans.inner_main_rect.bottom),
        &p,
    );
    for line in plans.horizontal_lines.iter() {
        canvas.draw_line(
            (layout_ctx.main_rect.left + 2.5, line.y_pos),
            (layout_ctx.main_rect.left + 17.5, line.y_pos),
            &p,
        );
    }
    p.set_anti_alias(true);

    p.set_style(PaintStyle::Fill);
    for section in plans.sections.sections.iter() {
        let (blob, point) = align_text(
            &section.text,
            &paint_ctx.fonts.small,
            (section.x + 5.0, layout_ctx.main_rect.bottom - 5.0),
            Align::BottomLeft,
        );
        canvas.draw_text_blob(blob, point, &p);
    }
    for line in plans.horizontal_lines.iter() {
        canvas.draw_text_align(
            format!("{}°", line.temperature),
            (layout_ctx.main_rect.left + 20.0, line.y_pos - 2.0),
            &paint_ctx.fonts.small,
            &p,
            skia_safe::utils::text_utils::Align::Right,
        );
    }

    p.set_style(PaintStyle::Stroke);

    p.set_stroke_width(3.0);
    if let Some(ref plan) = plans.temperature {
        canvas.draw_path(&plan.path, &p);
    }
    p.set_style(PaintStyle::Fill);
    if let Some(ref plan) = plans.rain {
        p.set_shader(plan.gradient.clone());
        canvas.draw_path(&plan.path, &p);
        p.set_shader(None);
    }
    p.set_style(PaintStyle::Stroke);
    if let Some(ref plan) = plans.p_precipitation {
        p.set_shader(plan.gradient.clone());
        canvas.draw_line(plan.points.0, plan.points.1, &p);
        p.set_shader(None);
    }
}

pub fn top(canvas: &Canvas, layout_ctx: &LayoutCtx, paint_ctx: &PaintCtx) {
    let mut p = Paint::default();
    p.set_style(PaintStyle::Fill);
    p.set_color(Color::from_rgb(0, 0, 0));

    let my_tz = jiff::tz::db().get("Europe/Berlin").unwrap();
    let now = jiff::Timestamp::now().to_zoned(my_tz);

    canvas.draw_text_align(
        icu_datetime::DateTimeFormatter::try_new(locale!("en-GB").into(), YMDE::long())
            .unwrap()
            .format(
                &Date::try_new_iso(now.year() as i32, now.month() as u8, now.day() as u8).unwrap(),
            )
            .to_string(),
        layout_ctx.top_rect.bottom_left() + Point::new(5.0, -5.0),
        &paint_ctx.fonts.big_bold,
        &p,
        skia_safe::utils::text_utils::Align::Left,
    );
}

pub fn bottom_left(canvas: &Canvas, plans: &Plans, layout_ctx: &LayoutCtx, paint_ctx: &PaintCtx) {
    let mut p = Paint::default();

    p.set_color(Color::from_rgb(0, 0, 0));
    p.set_style(PaintStyle::Fill);
    p.set_stroke_width(1.0);

    let (blob, origin) = align_text(
        "Outside",
        &paint_ctx.fonts.small,
        layout_ctx.bottom_left_rect.tr().moved(-5.0, 5.0),
        Align::TopRight,
    );
    canvas.draw_text_blob(blob, origin, &p);

    let (top_half, bottom_half) = layout_ctx
        .bottom_left_rect
        .y_split_frac(if plans.radar.is_some() { 0.6 } else { 1.0 });

    if let Some(ref current) = plans.current {
        let temp_y = if let Some(temp) = current.temperature {
            let (blob, origin) = align_text(
                &format!("{temp:.*}°", if temp.fract() == 0.0 { 0 } else { 1 }),
                &paint_ctx.fonts.big_bold,
                top_half.center(),
                Align::Bottom,
            );
            canvas.draw_text_blob(blob, origin, &p);
            Some(origin.y)
        } else {
            None
        };

        if let Some(wind) = current.mean_wind {
            let (blob, origin) = align_text(
                &format!("{wind:.*} km/h", if wind.fract() == 0.0 { 0 } else { 1 }),
                &paint_ctx.fonts.medium,
                if let Some(y) = temp_y {
                    Point::new(top_half.center_x(), y + 4.0)
                } else {
                    top_half.center()
                },
                if temp_y.is_some() {
                    Align::TopCenter
                } else {
                    Align::Center
                },
            );
            canvas.draw_text_blob(blob, origin, &p);
        }
    }

    if let Some(ref radar) = plans.radar {
        let bounds = Rect::from_ltrb(
            bottom_half.left + 20.0, // same as in the plan
            bottom_half.center_y() - 10.0,
            bottom_half.right - 20.0,
            bottom_half.center_y() + 10.0,
        );
        let rrect = RRect::new_rect_xy(bounds, 8.0, 8.0);

        p.set_shader(radar.shader.clone());
        p.set_color(Color::WHITE);
        canvas.draw_rrect(rrect, &p);
        p.set_shader(None);
        p.set_color(Color::BLACK);
        p.set_stroke_width(1.0);
        p.set_style(PaintStyle::Stroke);
        canvas.draw_rrect(rrect, &p);

        p.set_style(PaintStyle::Fill);

        let (blob, origin) = align_text(
            &radar.start_text.text,
            &paint_ctx.fonts.small,
            (radar.start_text.x_pos, bounds.bottom + 5.0),
            Align::TopCenter,
        );
        canvas.draw_text_blob(blob, origin, &p);

        let (blob, origin) = align_text(
            &radar.end_text.text,
            &paint_ctx.fonts.small,
            (radar.end_text.x_pos, bounds.bottom + 5.0),
            Align::TopCenter,
        );
        canvas.draw_text_blob(blob, origin, &p);

        if let Some(ref upper) = radar.upper_label {
            let (blob, origin) = align_text(
                &upper.text,
                &paint_ctx.fonts.small,
                (upper.x_pos, bounds.top - 5.0),
                Align::Bottom,
            );
            canvas.draw_text_blob(blob, origin, &p);

            canvas.draw_line(
                (upper.x_pos, bounds.top - 2.0),
                (upper.x_pos, bounds.bottom + 2.0),
                &p,
            );
        }
    }
}

pub fn bottom_right(
    canvas: &Canvas,
    layout_ctx: &LayoutCtx,
    paint_ctx: &PaintCtx,
    temp: f32,
    iaq: f32,
    co2: f32,
) {
    let mut p = Paint::default();

    p.set_color(Color::from_rgb(0, 0, 0));
    p.set_style(PaintStyle::Fill);
    p.set_stroke_width(1.0);

    let (blob, origin) = align_text(
        "Inside",
        &paint_ctx.fonts.small,
        layout_ctx.bottom_right_rect.tl().moved(5.0, 5.0),
        Align::TopLeft,
    );
    canvas.draw_text_blob(blob, origin, &p);

    let (blob, origin) = align_text(
        &format!("{temp:.*}°", if temp.fract() == 0.0 { 0 } else { 1 }),
        &paint_ctx.fonts.big_bold,
        layout_ctx.bottom_right_rect.center(),
        Align::Bottom,
    );
    canvas.draw_text_blob(blob, origin, &p);

    let (blob, origin) = align_text(
        &format!("IAQ: {iaq:.0}"),
        &paint_ctx.fonts.medium,
        Point::new(layout_ctx.bottom_right_rect.center_x(), origin.y + 4.0),
        Align::TopCenter,
    );
    canvas.draw_text_blob(blob, origin, &p);

    let (blob, origin) = align_text(
        &format!("CO2: {co2:.0} ppm"),
        &paint_ctx.fonts.medium,
        Point::new(layout_ctx.bottom_right_rect.center_x(), origin.y + 4.0),
        Align::TopCenter,
    );
    canvas.draw_text_blob(blob, origin, &p);
}
