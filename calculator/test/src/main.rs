use eframe::egui;
use eframe::egui::{vec2, Button, Ui, Window};
use std::sync::{Arc, Mutex};
use eval::Value;

// Define the Calculator struct
#[derive(Debug, Clone)]
pub struct Calculator {
    pub display: Arc<Mutex<String>>,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            display: Arc::new(Mutex::new("".to_string())),
        }
    }
}

// Implement Calculator logic
impl Calculator {
    fn add_digit(&self, digit: char) {
        let mut display = self.display.lock().unwrap();
        display.push(digit);
    }

    fn add_operator(&self, operator: char) {
        let mut display = self.display.lock().unwrap();
        let last_char = display.chars().last();

        // Allow operators after a closing parenthesis
        if !display.is_empty() && (last_char == Some(')') || !"+-*/()".contains(last_char.unwrap())) {
            display.push(operator);
        }
    }

    fn add_parenthesis(&self, parenthesis: char) {
        let mut display = self.display.lock().unwrap();
        let last_char = display.chars().last();

        if parenthesis == '(' {
            // Allow adding '(' if the display is empty or if the last character is an operator
            if display.is_empty() || "+-*/(".contains(last_char.unwrap()) {
                display.push(parenthesis);
            }
        } else if parenthesis == ')' {
            // Allow adding ')' if there's an open parenthesis and the last character isn't an operator
            let open_count = display.chars().filter(|&c| c == '(').count();
            let close_count = display.chars().filter(|&c| c == ')').count();

            if open_count > close_count && "+-*/0123456789".contains(last_char.unwrap()) {
                display.push(parenthesis);
            }
        }
    }

    fn calculate(&self) {
        let display = self.display.lock().unwrap().clone();
        let result = eval::eval(&display).unwrap_or_else(|_| Value::from(0.0));
        let mut display = self.display.lock().unwrap();
        *display = result.to_string();
    }

    fn clear(&self) {
        let mut display = self.display.lock().unwrap();
        display.clear();
    }

    fn backspace(&self) {
        let mut display = self.display.lock().unwrap();
        display.pop();
    }
}

// Define the CalculatorApp struct
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
        let window_margin = ui.spacing().window_margin;
        let size_1x1 = vec2(32.0, 26.0);

        ui.spacing_mut().item_spacing = vec2(window_margin.left, window_margin.top);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("7")).clicked() {
                    self.calculator.add_digit('7');
                }
                if ui.add_sized(size_1x1, Button::new("8")).clicked() {
                    self.calculator.add_digit('8');
                }
                if ui.add_sized(size_1x1, Button::new("9")).clicked() {
                    self.calculator.add_digit('9');
                }
                if ui.add_sized(size_1x1, Button::new("➗")).clicked() {
                    self.calculator.add_operator('/');
                }
                if ui.add_sized(size_1x1, Button::new("🔙")).clicked() {
                    self.calculator.backspace();
                }
            });
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("4")).clicked() {
                    self.calculator.add_digit('4');
                }
                if ui.add_sized(size_1x1, Button::new("5")).clicked() {
                    self.calculator.add_digit('5');
                }
                if ui.add_sized(size_1x1, Button::new("6")).clicked() {
                    self.calculator.add_digit('6');
                }
                if ui.add_sized(size_1x1, Button::new("✖")).clicked() {
                    self.calculator.add_operator('*');
                }
                if ui.add_sized(size_1x1, Button::new("🗑")).clicked() {
                    self.calculator.clear();
                }
            });
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("1")).clicked() {
                    self.calculator.add_digit('1');
                }
                if ui.add_sized(size_1x1, Button::new("2")).clicked() {
                    self.calculator.add_digit('2');
                }
                if ui.add_sized(size_1x1, Button::new("3")).clicked() {
                    self.calculator.add_digit('3');
                }
                if ui.add_sized(size_1x1, Button::new("➖")).clicked() {
                    self.calculator.add_operator('-');
                }
                if ui.add_sized(size_1x1, Button::new("=")).clicked() {
                    self.calculator.calculate();
                }
            });
            ui.horizontal(|ui| {
                if ui.add_sized(size_1x1, Button::new("(")).clicked() {
                    self.calculator.add_parenthesis('(');
                }
                if ui.add_sized(size_1x1, Button::new("0")).clicked() {
                    self.calculator.add_digit('0');
                }
                if ui.add_sized(size_1x1, Button::new(")")).clicked() {
                    self.calculator.add_parenthesis(')');
                }
                if ui.add_sized(size_1x1, Button::new("➕")).clicked() {
                    self.calculator.add_operator('+');
                }
                if ui.add_sized(size_1x1, Button::new(".")).clicked() {
                    self.calculator.add_digit('.');
                }
            });
        });
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Window::new("Calculator").show(ctx, |ui| {
            // Display the calculator's screen
            ui.horizontal(|ui| {
                ui.label(&*self.calculator.display.lock().unwrap());
            });

            ui.separator();

            // Display the calculator's keypad
            self.buttons(ui);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(CalculatorApp::default()))
        }),
    )
}
