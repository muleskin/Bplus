#![windows_subsystem = "windows"]
//! Bplus Rename: rename many files and folders at once, from one window.

mod favs;
mod engine;
mod files;
mod icon;
mod ui;

use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot = None;
    let mut start_dir = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            screenshot = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else {
            let p = PathBuf::from(&args[i]);
            if p.is_dir() {
                start_dir = Some(p);
            }
            i += 1;
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Bplus Rename")
            .with_inner_size([ui::WIN_W, ui::WIN_H])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(std::sync::Arc::new(egui::IconData { rgba: icon::rgba(64), width: 64, height: 64 })),
        ..Default::default()
    };
    eframe::run_native(
        "Bplus Rename",
        options,
        Box::new(move |cc| Ok(Box::new(ui::App::new(&cc.egui_ctx, start_dir, screenshot)))),
    )
}
