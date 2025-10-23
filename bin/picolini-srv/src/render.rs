use anyhow::{anyhow, bail};
use dwd_fetch::Cache;
use skia_safe::{
    Color, Color4f, ColorInfo, EncodedImageFormat, Image, ImageInfo, Paint, Pixmap, surfaces,
};

use crate::{
    EPD_HEIGHT, EPD_WIDTH,
    layout_ctx::LayoutCtx,
    paint::{self, PaintCtx},
    plan::Plans,
};

pub fn to_dithered_png(
    cache: &Cache,
    paint_ctx: &PaintCtx,
    temp: f32,
    iaq: f32,
    co2: f32,
) -> anyhow::Result<Box<[u8]>> {
    let image = to_image(cache, paint_ctx, temp, iaq, co2)?;
    let mut pixmap = image
        .peek_pixels()
        .ok_or_else(|| anyhow!("Can't peek pixels"))?;
    let _ = dither_pixmap(&mut pixmap);
    let Some(png) = pixmap.encode(EncodedImageFormat::PNG, None) else {
        bail!("Failed to encode image");
    };
    Ok(png.into_boxed_slice())
}

pub fn full(
    cache: &Cache,
    paint_ctx: &PaintCtx,
    temp: f32,
    iaq: f32,
    co2: f32,
) -> anyhow::Result<Box<[u8]>> {
    let image = to_image(cache, paint_ctx, temp, iaq, co2)?;
    let mut pixmap = image
        .peek_pixels()
        .ok_or_else(|| anyhow!("Can't peek pixels"))?;
    Ok(dither_pixmap(&mut pixmap))
}

fn to_image(
    cache: &Cache,
    paint_ctx: &PaintCtx,
    temp: f32,
    iaq: f32,
    co2: f32,
) -> anyhow::Result<Image> {
    let mut surface = surfaces::raster(
        &ImageInfo::from_color_info(
            (EPD_WIDTH, EPD_HEIGHT),
            ColorInfo::new(
                skia_safe::ColorType::Gray8,
                skia_safe::AlphaType::Opaque,
                None,
            ),
        ),
        None,
        None,
    )
    .unwrap();
    let canvas = surface.canvas();
    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

    let mut p = Paint::default();
    p.set_color(Color::from_rgb(0, 0, 0));
    p.set_stroke(true);
    p.set_stroke_width(1.0);

    let layout_ctx = LayoutCtx::default();

    canvas.draw_line(
        layout_ctx.mid_bottom_divider.0,
        layout_ctx.mid_bottom_divider.1,
        &p,
    );
    canvas.draw_line(layout_ctx.bottom_divider.0, layout_ctx.bottom_divider.1, &p);

    let plans = Plans::new(cache, &layout_ctx);

    paint::main(canvas, &plans, &layout_ctx, paint_ctx);
    paint::top(canvas, &layout_ctx, paint_ctx);
    paint::bottom_left(canvas, &plans, &layout_ctx, paint_ctx);
    paint::bottom_right(canvas, &layout_ctx, paint_ctx, temp, iaq, co2);

    Ok(surface.image_snapshot())
}

fn dither_pixmap(pix: &mut Pixmap) -> Box<[u8]> {
    let (w, h) = (pix.width(), pix.height());
    let pixels = pix.bytes_mut().unwrap();
    let mut epd = vec![0u8; EPD_WIDTH as usize / 8 * EPD_HEIGHT as usize].into_boxed_slice();
    for i in 0..h {
        for j in 0..w {
            let off = (i * w + j) as usize;
            let old = pixels[off];
            let new = if old < 128 {
                0u8
            } else {
                set_epd_pixel(&mut epd, j, i);
                255
            };
            pixels[off] = new;
            let my_err = old as i16 - new as i16;

            if j < w - 1 {
                pixels[off + 1] = (pixels[off + 1] as i16 + my_err * 7 / 16).clamp(0, 255) as u8;
            }
            if i < h - 1 {
                if j > 0 {
                    let off = ((i + 1) * w + j - 1) as usize;
                    pixels[off] = (pixels[off] as i16 + my_err * 3 / 16).clamp(0, 255) as u8;
                }
                let off = ((i + 1) * w + j) as usize;
                pixels[off] = (pixels[off] as i16 + my_err * 5 / 16).clamp(0, 255) as u8;
                if j < w - 1 {
                    let off = ((i + 1) * w + j + 1) as usize;
                    pixels[off] = (pixels[off] as i16 + my_err / 16).clamp(0, 255) as u8;
                }
            }
        }
    }
    epd
}

fn set_epd_pixel(buf: &mut [u8], x: i32, y: i32) {
    let index = x as usize / 8 + y as usize * (EPD_WIDTH as usize / 8);
    let bit = (0b1000_0000 >> (x % 8)) as u8;
    let mask = !bit;
    buf[index] = buf[index] & mask | bit;
}
