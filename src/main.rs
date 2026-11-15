#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod install;
mod watcher;

use std::path::PathBuf;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let arg = std::env::args_os().nth(1);

    if arg.as_deref() == Some(std::ffi::OsStr::new("--install")) {
        return install::run();
    }

    let path = arg.map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("MD Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "MD Viewer",
        options,
        Box::new(move |cc| Ok(Box::new(app::MdViewer::new(cc, path)))),
    )
    .context("failed to start the GUI")
}
