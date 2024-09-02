use egui::{Button, Id, Rgba};
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
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../data/Vazir-Bold-FD-WOL.ttf"
                )),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "my_font".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("my_font".to_owned());

            ctx.set_fonts(fonts);

            let combined_table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::initial(150.0).at_least(25.0))
                .min_scrolled_height(0.0);

            combined_table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("test");
                    });
                })
                .body(|mut body| {
                    body.row(50.0, |mut row| {
                        row.col(|ui| {
                            ui.add_sized(vec2(150.0, 50.0), Button::new("test"));
                        });
                    })
                })
        });
    }
}