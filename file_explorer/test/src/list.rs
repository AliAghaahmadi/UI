use std::path::Path;
use eframe::epaint::Color32;
use egui::{popup_above_or_below_widget, AboveOrBelow, Id, PopupCloseBehavior, RichText, Ui};
use crate::explorer::FileBrowserApp;

pub fn list_explorer(app: &mut FileBrowserApp, mut ui: &mut Ui)
{
    let mut new_path = None;

    let combined_table = egui_extras::TableBuilder::new(&mut ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::initial(100.0).at_least(25.0))
        .min_scrolled_height(0.0);

    combined_table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Name");
            });
        })
        .body(|mut body| {
            for directory in &mut app.directories {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("📁");
                        let dir = ui.button(&directory.name);

                        if dir.clicked() {
                            new_path = Some(directory.dir.clone());
                        }

                        let id = Id::new(format!("1 {}", &directory.name));

                        if dir.secondary_clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(id));
                        }

                        popup_above_or_below_widget(
                            ui,
                            id,
                            &dir,
                            AboveOrBelow::Above,
                            PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.set_min_width(100.0);
                                let size = directory.size.lock().unwrap().clone();
                                let calculating = *directory.calculating.lock().unwrap();
                                let error = directory.error.lock().unwrap().clone();

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Name: ");
                                        ui.strong(directory.clone().name);
                                    });
                                    if error.is_some() {
                                        ui.horizontal(|ui| {
                                            ui.label("Error: ");
                                            ui.label(RichText::new("Something Went Wrong").color(Color32::RED));
                                        });
                                    } else if size.is_none() && !calculating {
                                        ui.horizontal(|ui| {
                                            ui.label("Size: ");
                                            FileBrowserApp::directory_size(directory);
                                        });
                                    } else if calculating {
                                        ui.horizontal(|ui| {
                                            ui.label("Size: ");
                                            ui.add(egui::Spinner::new());
                                            ui.label("Calculating...");
                                        });
                                    } else {
                                        ui.horizontal(|ui| {
                                            ui.label("Size: ");
                                            ui.label(FileBrowserApp::format_size(size));
                                        });
                                    }
                                })
                            },
                        );
                    });
                });
            }

            for file in &app.files {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        let path = Path::new(&file.name);
                        if path.extension() != None
                        { ui.label(FileBrowserApp::extension_icon(path.extension().unwrap().to_str().unwrap()).unwrap().to_string()); }
                        else { ui.label("❓"); }
                        let file_btn = ui.button(&file.name);

                        if file_btn.clicked() {
                            //new_path = Some(file.dir.clone());
                        }

                        let id = Id::new(format!("2 {}", &file.name));

                        if file_btn.secondary_clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(id));
                        }

                        popup_above_or_below_widget(
                            ui,
                            id,
                            &file_btn,
                            AboveOrBelow::Above,
                            PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.set_min_width(100.0);
                                if let Some(size) = file.size {
                                    ui.label(format!("Size: {}", FileBrowserApp::format_size(Some(size))));
                                } else {
                                    ui.label("Size unknown");
                                }
                            },
                        );
                    });
                });
            }
        });

    if let Some(path) = new_path {
        app.current_path = path;
        app.search = "".to_string();
        app.update_directory_list(&app.current_path.clone());
    }
}