#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use clap::Parser;
use skia_safe::{Data, EncodedImageFormat, Image, surfaces};
use std::fs;
use std::path::PathBuf;

use crate::application::Application;
use crate::config::CONFIG;
use crate::context::Context;
use crate::paint::{PaintCtx, Pipeline};

mod application;
mod config;
mod context;
mod extensions;
mod icons;
mod layout;
mod paint;
mod picolini;
mod platform;
mod window;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run the app in a movable window
    Windowed,
    /// Show the result as the desktop background
    Background,
    /// Export the current view as a PNG
    Export {
        #[arg(short, long, default_value = "out.png")]
        output: PathBuf,
    },
    /// Add the app to the autostart
    Autostart {
        #[clap(subcommand)]
        command: AutostartCommand,
    },
}

#[derive(clap::Subcommand)]
enum AutostartCommand {
    Add,
    Remove,
}

fn cd_to_exe() {
    std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap()).unwrap();
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(Command::Autostart { command }) = &args.command {
        match command {
            AutostartCommand::Add => platform::windows::add_self_to_autostart()?,
            AutostartCommand::Remove => platform::windows::remove_autostart()?,
        }
        return Ok(());
    }

    cd_to_exe();

    let bg_img = Image::from_encoded(
        Data::from_filename("bg.png").ok_or_else(|| anyhow!("Failed to read bg.png"))?,
    )
    .ok_or_else(|| anyhow!("Failed to read bg.png as image"))?;

    let mut context = Context::new(
        dwd_fetch::Cache::from_file_or_default(CONFIG.cache_file()),
        bg_img,
    );
    if !context.update() {
        context.replan();
    }

    match args.command {
        None | Some(Command::Background) => Windowed {
            as_background: true,
        }
        .run(context),
        Some(Command::Windowed) => Windowed {
            as_background: false,
        }
        .run(context),
        Some(Command::Export { output }) => ToImage { output }.run(context),
        Some(Command::Autostart { .. }) => unreachable!(),
    }

    Ok(())
}

trait Frontend {
    fn run(self, context: Context);
}

#[allow(unused)]
struct Windowed {
    as_background: bool,
}

#[allow(unused)]
struct ToImage {
    output: PathBuf,
}

impl Frontend for Windowed {
    fn run(self, mut context: Context) {
        let mut pipl = Pipeline::default();
        context.relayout(&mut pipl);
        Application::new(context, pipl, self.as_background).run();
    }
}

impl Frontend for ToImage {
    fn run(self, mut context: Context) {
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
        fs::write(self.output, data.as_bytes()).unwrap();
    }
}
