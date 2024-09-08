use eframe::egui::{Button, Ui, Vec2};

pub struct Keypad {
    pub cursor_pos: usize,
    pub cursor_right: bool,
    pub cursor_left: bool,
    pub done: bool,
}

impl Keypad {
    pub fn new() -> Self {
        Self {
            cursor_pos: 0,
            cursor_right: false,
            cursor_left: false,
            done: false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, input: &mut String) {
        let size_1x1 = Vec2::new(32.0, 26.0);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("1")).clicked() {
                    self.insert_text(input, "1");
                }
                if ui.add_sized(size_1x1, Button::new("2")).clicked() {
                    self.insert_text(input, "2");
                }
                if ui.add_sized(size_1x1, Button::new("3")).clicked() {
                    self.insert_text(input, "3");
                }
                if ui.add_sized(size_1x1, Button::new("🔙")).clicked() {
                    self.remove_char(input);
                }
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("4")).clicked() {
                    self.insert_text(input, "4");
                }
                if ui.add_sized(size_1x1, Button::new("5")).clicked() {
                    self.insert_text(input, "5");
                }
                if ui.add_sized(size_1x1, Button::new("6")).clicked() {
                    self.insert_text(input, "6");
                }
                if ui.add_sized(size_1x1, Button::new("➡")).clicked() {
                    self.cursor_right = true;
                }
                if ui.button("⎆").clicked() {
                    self.done = true;
                }
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("7")).clicked() {
                    self.insert_text(input, "7");
                }
                if ui.add_sized(size_1x1, Button::new("8")).clicked() {
                    self.insert_text(input, "8");
                }
                if ui.add_sized(size_1x1, Button::new("9")).clicked() {
                    self.insert_text(input, "9");
                }
                if ui.add_sized(size_1x1, Button::new("➖")).clicked() {
                    self.insert_text(input, "-");
                }
                if ui.add_sized(size_1x1, Button::new("⬅")).clicked() {
                    self.cursor_left = true;
                }
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("0")).clicked() {
                    self.insert_text(input, "0");
                }
                if ui.add_sized(size_1x1, Button::new(".")).clicked() {
                    self.insert_text(input, ".");
                }
                if ui.add_sized(size_1x1, Button::new("✖")).clicked() {
                    self.insert_text(input, "*");
                }
                if ui.add_sized(size_1x1, Button::new("➗")).clicked() {
                    self.insert_text(input, "/");
                }
            });
            ui.add_space(5.0);
        });
    }

    fn insert_text(&mut self, input: &mut String, text: &str) {
        let pos = self.cursor_pos.min(input.len());
        if pos <= input.len() {
            input.insert_str(pos, text);
            self.cursor_pos = (pos + text.len()).min(input.len());
            self.cursor_right = true;
            self.cursor_left = false;
        }
    }

    fn remove_char(&mut self, input: &mut String) {
        if self.cursor_pos > 0 {
            let pos = self.cursor_pos - 1;
            if pos < input.len() {
                input.remove(pos);
                self.cursor_pos = pos;
                self.cursor_right = false;
                self.cursor_left = true;
            }
        }
    }
}
