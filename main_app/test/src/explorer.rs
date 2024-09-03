use eframe::epaint::{Color32, Rounding};
use egui::{Button, FontId};
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
                    "../data/ARIAL.TTF"
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
                            let color1 = Color32::from_rgba_premultiplied(120, 120, 120, 204);
                            let color2 = Color32::from_rgba_premultiplied(0, 0, 0, 0);

                            let color = if self.file_exp_open { color1 } else { color2 };

                            let button_text = egui::RichText::new("😃").font(FontId::proportional(25.0));
                            set_widget_rounding(ui.style_mut(), 0.0, 0.0, 25.0, 25.0);

                            if ui.add_sized(vec2(150.0, 50.0), Button::new(button_text).fill(color)).clicked()
                            {
                                self.file_exp_open = !self.file_exp_open;
                            }
                        });
                    })
                })
        });
    }
}


fn set_widget_rounding(style: &mut egui::Style, rounding1: f32, rounding2: f32, rounding3: f32, rounding4: f32) {
    let rounding = Rounding {
        nw: rounding1,
        ne: rounding2,
        sw: rounding3,
        se: rounding4,
    };

    style.visuals.widgets.inactive.rounding = rounding;
    style.visuals.widgets.active.rounding = rounding;
    style.visuals.widgets.hovered.rounding = rounding;
}