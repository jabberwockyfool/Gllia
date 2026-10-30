// Pas de console noire derrière la fenêtre en release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod catalogue;
mod config;
mod fonts;
mod install;
mod layouts;
mod link;
mod music;
mod palette;
mod screens;
mod stores;
mod update;
mod windows;
mod xdelta;

use app::LauncherApp;

fn main() -> eframe::Result {
    let link = std::env::args().nth(1).and_then(|arg| link::parse(&arg));
    if link.as_deref().is_some_and(link::send_to_running) {
        return Ok(());
    }
    let _ = link::register();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Gllia")
        .with_inner_size([1100.0, 650.0])
        .with_min_inner_size([800.0, 500.0]);

    let icon = assets::embedded("icon.png").and_then(|bytes| eframe::icon_data::from_png_bytes(bytes).ok());
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Gllia",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            fonts::install(&cc.egui_ctx);
            Ok(Box::new(LauncherApp::new(&cc.egui_ctx, link)))
        }),
    )
}
