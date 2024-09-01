use eframe::egui;
use eframe::egui::{Button, Vec2};

pub struct CalculatorApp {
    pub input: String,
}

impl Default for CalculatorApp {
    fn default() -> Self {
        Self {
            input: String::new(),
        }
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Calculator");

            // Text edit widget to display and edit the input
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.input);
            });

            ui.separator();

            let size_1x1 = Vec2::new(32.0, 26.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(4.0);

                ui.horizontal(|ui| {
                    if ui.add_sized(size_1x1, Button::new("1")).clicked() {
                        self.input.push('1');
                    }
                    if ui.add_sized(size_1x1, Button::new("2")).clicked() {
                        self.input.push('2');
                    }
                    if ui.add_sized(size_1x1, Button::new("3")).clicked() {
                        self.input.push('3');
                    }
                    if ui.add_sized(size_1x1, Button::new("⏮")).clicked() {
                        // Handle Home key behavior if needed
                    }
                    if ui.add_sized(size_1x1, Button::new("🔙")).clicked() {
                        self.input.pop(); // Handle Backspace behavior
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add_sized(size_1x1, Button::new("4")).clicked() {
                        self.input.push('4');
                    }
                    if ui.add_sized(size_1x1, Button::new("5")).clicked() {
                        self.input.push('5');
                    }
                    if ui.add_sized(size_1x1, Button::new("6")).clicked() {
                        self.input.push('6');
                    }
                    if ui.add_sized(size_1x1, Button::new("⏭")).clicked() {
                        // Handle End key behavior if needed
                    }
                    if ui.add_sized(size_1x1, Button::new("⎆")).clicked() {
                        // Handle Enter key behavior if needed
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add_sized(size_1x1, Button::new("7")).clicked() {
                        self.input.push('7');
                    }
                    if ui.add_sized(size_1x1, Button::new("8")).clicked() {
                        self.input.push('8');
                    }
                    if ui.add_sized(size_1x1, Button::new("9")).clicked() {
                        self.input.push('9');
                    }
                    if ui.add_sized(size_1x1, Button::new("⏶")).clicked() {
                        // Handle ArrowUp key behavior if needed
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add_sized(size_1x1, Button::new("0")).clicked() {
                        self.input.push('0');
                    }
                    if ui.add_sized(size_1x1, Button::new(".")).clicked() {
                        self.input.push('.');
                    }
                    if ui.add_sized(size_1x1, Button::new("⏴")).clicked() {
                        // Handle ArrowLeft key behavior if needed
                    }
                    if ui.add_sized(size_1x1, Button::new("⏷")).clicked() {
                        // Handle ArrowDown key behavior if needed
                    }
                    if ui.add_sized(size_1x1, Button::new("⏵")).clicked() {
                        // Handle ArrowRight key behavior if needed
                    }
                });
            });
        });
    }
}
