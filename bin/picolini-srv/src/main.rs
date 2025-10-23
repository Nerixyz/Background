use std::sync::{Arc, RwLock};

use dwd_fetch::Cache;

use crate::{config::CONFIG, fonts::Fonts, paint::PaintCtx};

mod config;
mod fonts;
mod layout_ctx;
mod paint;
mod plan;
mod render;
mod web;

pub const EPD_WIDTH: i32 = 648;
pub const EPD_HEIGHT: i32 = 480;

fn main() -> std::io::Result<()> {
    if std::env::args().nth(1).is_some_and(|it| it == "render") {
        one_off_render();
        return Ok(());
    }
    env_logger::init();
    web::main()
}

fn one_off_render() {
    let cache = Arc::new(RwLock::new(Cache::from_file_or_default("cache.bin")));
    Cache::refetch(&cache, CONFIG.dwd()).unwrap();

    let paint_ctx = PaintCtx {
        fonts: Fonts::new("./fonts/InterVariable.ttf"),
    };
    let data =
        render::to_dithered_png(&cache.read().unwrap(), &paint_ctx, 20.3, 50.0, 400.0).unwrap();
    std::fs::write("epd.png", &data).unwrap()
}
