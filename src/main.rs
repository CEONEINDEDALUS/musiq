mod app;
mod audio;
mod library;
mod persist;
mod search;
mod theme;
mod ui;

use app::MusiqApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 600.0])
            .with_title("musiq")
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "musiq",
        options,
        Box::new(|cc| {
            crate::theme::apply(&cc.egui_ctx);
            Ok(Box::new(MusiqApp::new(cc)))
        }),
    )
}
