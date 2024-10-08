use eframe::egui;
use crate::explorer::load_style_from_file;

mod explorer;
mod list;
// Import the file_browser module

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "File Browser",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::<explorer::FileBrowserApp>::default())
        }),
    )
}
