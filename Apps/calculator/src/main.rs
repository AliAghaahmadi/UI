mod calculator;

use eframe::egui;
use calculator::Keypad;
use eframe::egui::{Button, Color32, Key, Response, RichText, TextEdit, Ui};
use egui_extras::TableBuilder;
use fend_core;
use fend_core::Context;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350.0, 450.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Custom Keypad App",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Debug)]
struct Calculation {
    input: String,
    output: String,
    done: bool,
}

struct MyApp {
    input: String,
    keypad: Keypad,
    keypad_open: bool,
    context: Context,
    textedit: Option<Response>,
    calculations: Vec<Calculation>,
}

impl MyApp {
    fn done(&mut self) {
        if !self.input.is_empty() {
            match fend_core::evaluate(&self.input, &mut self.context) {
                Ok(evaluation) => {
                    self.calculations.push(Calculation {
                        input: self.input.clone(),
                        output: evaluation.get_main_result().to_string(),
                        done: true,
                    });
                }
                Err(_) => {
                    self.calculations.push(Calculation {
                        input: self.input.clone(),
                        output: "Not Complete".to_string(),
                        done: true,
                    });
                }
            }

            self.input.clear();
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            keypad: Keypad::new(),
            keypad_open: false,
            calculations: vec![],
            textedit: None,
            context: Context::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.group(|ui| {
                let calculations_table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::initial(20.0))
                    .column(egui_extras::Column::initial(150.0))
                    .column(egui_extras::Column::initial(100.0))
                    .max_scroll_height(50.0);

                let total_rows = self.calculations.len().max(9);

                // Build the calculations table
                calculations_table.body(|mut body| {
                    for index in 0..total_rows {
                        body.row(20.0, |mut row| {
                            if let Some(calculation) = self.calculations.get(index) {
                                row.col(|ui| {
                                    ui.label(RichText::new(format!("{}", index + 1)).color(Color32::LIGHT_BLUE));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", calculation.input.clone()));
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(calculation.output.clone()).color(Color32::LIGHT_GREEN));
                                });
                            } else {
                                // Empty rows
                                row.col(|ui| {
                                    ui.label("");
                                });
                                row.col(|ui| {
                                    ui.label("");
                                });
                                row.col(|ui| {
                                    ui.label("");
                                });
                            }
                        });
                    }
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    let current_row = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(egui_extras::Column::exact(20.0))
                        .column(egui_extras::Column::exact(150.0))
                        .column(egui_extras::Column::exact(100.0))
                        .max_scroll_height(20.0);

                    current_row.body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                toggle_button("🖩", &mut self.keypad_open, ui);
                            });
                            row.col(|ui| {
                                self.textedit = Some(ui.add(TextEdit::singleline(&mut self.input).desired_width(150.0)));
                            });
                            row.col(|ui| {
                                if let Ok(evaluation) = fend_core::evaluate(&self.input, &mut self.context) {
                                    ui.label(RichText::new(evaluation.get_main_result()).color(Color32::GREEN));
                                }
                            });
                        });
                    });
                });


                if ctx.input(|i| i.key_down(Key::Enter)) {
                    if let Some(textedit) = &self.textedit {
                        textedit.request_focus();
                    }
                    self.done();
                }

                if self.keypad_open {
                    egui::Window::new("Custom Keypad")
                        .fixed_pos([5.0, 260.0])
                        .resizable(false)
                        .title_bar(false)
                        .show(ctx, |ui| {
                            self.keypad.show(ui, &mut self.input);
                        });

                    if self.keypad.done {
                        if let Some(textedit) = &self.textedit {
                            textedit.request_focus();
                        }
                        self.done();
                        self.keypad.done = false;
                    }

                    let textedit_id = self.textedit.clone().unwrap().id;

                    if let Some(mut state) = TextEdit::load_state(ctx, textedit_id) {
                        if let Some(range) = state.cursor.char_range() {
                            self.keypad.cursor_pos = range.primary.index;
                        }

                        if self.keypad.cursor_right {
                            self.keypad.cursor_pos = (self.keypad.cursor_pos + 1).min(self.input.len());
                            self.keypad.cursor_right = false;
                        }

                        if self.keypad.cursor_left {
                            self.keypad.cursor_pos = self.keypad.cursor_pos.saturating_sub(1);
                            self.keypad.cursor_left = false;
                        }

                        let cursor = egui::text::CCursor::new(self.keypad.cursor_pos);
                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                        state.store(ctx, textedit_id);
                        if let Some(textedit) = &self.textedit {
                            textedit.request_focus();
                        }
                    }
                }
            });
        });
    }
}

fn toggle_button(text: &str, toggle: &mut bool, ui: &mut Ui) {
    let color = if *toggle {
        ui.style().visuals.selection.bg_fill
    } else {
        ui.style().visuals.widgets.inactive.weak_bg_fill
    };

    if ui.add(Button::new(text).fill(color)).clicked() {
        *toggle = !*toggle;
    }
}
