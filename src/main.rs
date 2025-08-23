#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use skia_safe::{Data, EncodedImageFormat, Image, surfaces};
use std::fs;

use crate::application::Application;
use crate::config::CONFIG;
use crate::context::Context;
use crate::paint::{PaintCtx, Pipeline};

mod application;
mod config;
mod context;
mod dwd;
mod extensions;
mod graph;
mod graphics;
mod icons;
mod layout;
mod paint;
mod platform;
mod squircle;
mod window;

fn main() -> anyhow::Result<()> {
    let bg_img = Image::from_encoded(
        Data::from_filename("bg.png").ok_or_else(|| anyhow!("Failed to read bg.png"))?,
    )
    .ok_or_else(|| anyhow!("Failed to read bg.png as image"))?;

    let mut context = Context::new(
        dwd::Cache::from_file_or_default(CONFIG.cache_file()),
        bg_img,
    );
    if !context.update() {
        context.replan();
    }

    Windowed::run(context);

    Ok(())
}

trait Frontend {
    fn run(context: Context);
}

#[allow(unused)]
struct Windowed;

#[allow(unused)]
struct ToImage;

impl Frontend for Windowed {
    fn run(mut context: Context) {
        let mut pipl = Pipeline::default();
        context.relayout(&mut pipl);
        Application::new(context, pipl).run();
    }
}

impl Frontend for ToImage {
    fn run(mut context: Context) {
        let mut surface =
            surfaces::raster_n32_premul((context.bg_image.width(), context.bg_image.height()))
                .unwrap();
        let canvas = surface.canvas();

        let mut pipl = Pipeline::default();
        context.relayout(&mut pipl);

        let mut paint_ctx = PaintCtx::new(&context.layout_ctx);
        pipl.paint(canvas, &mut paint_ctx);

        let image = surface.image_snapshot();
        let data = image
            .encode(
                surface.direct_context().as_mut(),
                EncodedImageFormat::PNG,
                None,
            )
            .unwrap();
        fs::write("out.png", data.as_bytes()).unwrap();
    }
}
