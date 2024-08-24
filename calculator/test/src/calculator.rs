// calculator.rs

use crate::egui;
use eframe::egui::{vec2, Button, Ui, Vec2, Window};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Calculator {
    pub input: Arc<Mutex<String>>,
    pub result: String,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            input: Arc::new(Mutex::new("".to_string())),
            result: String::new(),
        }
    }
}

impl Calculator {
    pub fn add_input(&self, c: char) {
        let mut lock = self.input.lock().unwrap();
        lock.push(c);
    }

    pub fn clear_input(&self) {
        let mut lock = self.input.lock().unwrap();
        lock.clear();
    }

    pub fn backspace(&self) {
        let mut lock = self.input.lock().unwrap();
        if !lock.is_empty() {
            lock.pop();
        }
    }
}

pub struct CalculatorApp {
    pub calculator: Calculator,
}

impl Default for CalculatorApp {
    fn default() -> Self {
        Self {
            calculator: Calculator::default(),
        }
    }
}

impl CalculatorApp {
    fn buttons(&mut self, ui: &mut Ui) {
        let size_1x1 = vec2(32.0, 32.0);

        ui.spacing_mut().item_spacing = Vec2::splat(5.0);

        ui.horizontal(|ui| {
            if ui.add_sized(size_1x1, Button::new("7")).clicked() { self.calculator.add_input('7'); }
            if ui.add_sized(size_1x1, Button::new("8")).clicked() { self.calculator.add_input('8'); }
            if ui.add_sized(size_1x1, Button::new("9")).clicked() { self.calculator.add_input('9'); }
            if ui.add_sized(size_1x1, Button::new("➗")).clicked() { self.calculator.add_input('/'); }
            if ui.add_sized(size_1x1, Button::new("🔙")).clicked() { self.calculator.backspace() }
        });
        ui.horizontal(|ui| {
            if ui.add_sized(size_1x1, Button::new("4")).clicked() { self.calculator.add_input('4'); }
            if ui.add_sized(size_1x1, Button::new("5")).clicked() { self.calculator.add_input('5'); }
            if ui.add_sized(size_1x1, Button::new("6")).clicked() { self.calculator.add_input('6'); }
            if ui.add_sized(size_1x1, Button::new("✖")).clicked() { self.calculator.add_input('*'); }
            if ui.add_sized(size_1x1, Button::new("🗑")).clicked() { self.calculator.clear_input(); }
        });
        ui.horizontal(|ui| {
            if ui.add_sized(size_1x1, Button::new("1")).clicked() { self.calculator.add_input('1'); }
            if ui.add_sized(size_1x1, Button::new("2")).clicked() { self.calculator.add_input('2'); }
            if ui.add_sized(size_1x1, Button::new("3")).clicked() { self.calculator.add_input('3'); }
            if ui.add_sized(size_1x1, Button::new("➖")).clicked() { self.calculator.add_input('-'); }
            if ui.add_sized(size_1x1, Button::new("=")).clicked() { self.calculator.add_input('='); }
        });
        ui.horizontal(|ui| {
            if ui.add_sized(size_1x1, Button::new("(")).clicked() { self.calculator.add_input('('); }
            if ui.add_sized(size_1x1, Button::new("0")).clicked() { self.calculator.add_input('0'); }
            if ui.add_sized(size_1x1, Button::new(")")).clicked() { self.calculator.add_input(')'); }
            if ui.add_sized(size_1x1, Button::new("➕")).clicked() { self.calculator.add_input('+'); }
            if ui.add_sized(size_1x1, Button::new(".")).clicked() { self.calculator.add_input('.'); }
        });
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Window::new("Calculator").show(ctx, |ui| {
            ui.add_space(50.0);

            // Display the calculator's screen
            ui.horizontal(|ui| {
                ui.label(&*self.calculator.input.lock().unwrap());
            });

            ui.separator();

            self.buttons(ui);

            ui.horizontal(|ui| {
                ui.label(&*self.calculator.result);
            });
        });
    }
}
