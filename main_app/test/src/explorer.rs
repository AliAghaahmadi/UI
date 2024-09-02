use egui::Button;
use eframe::epaint::Color32;
use egui::vec2;

pub struct MainApp {
    wifi_open: bool,
    file_exp_open: bool,
}

impl Default for MainApp {
    fn default() -> Self {
        let mut app = Self {
            wifi_open: false,
            file_exp_open: false,
        };
        app
    }
}

impl MainApp {

}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.scope(|ui| {
                let mut test = Color32::BLACK;

                if self.file_exp_open { test = Color32::BLACK }
                else { test = Color32::RED }

                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = test;

                if  ui.add_sized(vec2(36.0, 25.0), Button::new("test")).clicked() { self.file_exp_open = !self.file_exp_open }
            });
        });
    }
}