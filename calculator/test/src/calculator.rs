use std::collections::HashMap;
use eframe::egui;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui::TextWrapMode::Wrap;

// Define the FileBrowserApp struct
pub struct FileBrowserApp {
    pub current: String,
}

impl Default for FileBrowserApp {
    fn default() -> Self {
        let mut app = Self {
            current: "".to_string(),
        };

        app
    }
}

impl FileBrowserApp {
    pub fn update_directory_list(&mut self, path: &str) {
        self.files.clear();
        self.directories.clear();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(dir_name) = path.file_name() {
                            let dir_name = dir_name.to_string_lossy().to_string();
                            let dir_path = path.to_string_lossy().to_string();
                            self.directories.push(Folder {
                                dir: dir_path,
                                name: dir_name,
                                size: Arc::new(Mutex::new(None)),
                                calculating: Arc::new(Mutex::new(false)),
                            });
                        }
                    } else {
                        if let Some(file_name) = path.file_name() {
                            let file_name = file_name.to_string_lossy().to_string();
                            let file_path = path.to_string_lossy().to_string();
                            let file = File {
                                dir: file_path,
                                name: file_name,
                                size: metadata(&path).ok().map(|m| m.len()),
                            };
                            self.files.push(file);
                        }
                    }
                }
            }
        }
    }
}

fn get_parent_directories(path: &Path) -> Vec<PathBuf> {
    let mut parents = Vec::new();
    let mut current_path = path.to_path_buf();
    parents.push(current_path.clone());

    while let Some(parent) = current_path.parent() {
        if parent == Path::new("") {
            break;
        }
        parents.push(parent.to_path_buf());
        current_path = parent.to_path_buf();
    }

    parents.reverse();
    parents
}

impl eframe::App for FileBrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Browser");

            ui.horizontal(|ui| {
                if ui.button("⏴").clicked() {
                    if let Some(parent) = Path::new(&self.current_path).parent() {
                        if let Some(parent_str) = parent.to_str() {
                            let parent_path = parent_str.to_string();
                            self.current_path = parent_path.clone();
                            self.update_directory_list(&parent_path);
                        }
                    }
                }

                ui.horizontal(|ui|{
                    for parents in get_parent_directories(Path::new(&self.current_path)){
                        if parents.file_name() != None
                        {
                            if ui.button(parents.file_name().unwrap().to_string_lossy()).clicked()
                            {
                                self.current_path = parents.to_string_lossy().parse().unwrap();
                                self.update_directory_list(&*parents.to_string_lossy());
                            }

                            ui.label("/");
                        }
                    }
                })
            });
            ui.separator();

            ui.label("Directories & Files:");
            let mut new_path = None;
            let combined_table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::initial(150.0).at_least(25.0))
                .column(egui_extras::Column::initial(150.0).at_least(25.0))
                .min_scrolled_height(0.0);

            combined_table.header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Size");
                });
            })
                .body(|mut body| {
                    for directory in &mut self.directories {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("📁");
                                if ui.button(&directory.name).clicked() {
                                    new_path = Some(format!("{}/{}", self.current_path, directory.name));
                                }
                            });

                            row.col(|ui| {
                                let size = directory.size.lock().unwrap().clone();
                                let calculating = *directory.calculating.lock().unwrap();

                                if size.is_none() && !calculating {
                                    if ui.button("Calculate").clicked() {
                                        directory_size(directory);
                                    }
                                } else if calculating {
                                    ui.add(egui::Spinner::new());
                                    ui.label("Calculating...");
                                } else {
                                    ui.label(format_size(size));
                                }
                            });
                        });
                    }

                    for file in &self.files {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                if Path::new(&file.clone().dir).extension() != None {
                                    ui.label(extension_icon(&Path::new(&file.clone().dir).extension().unwrap().to_string_lossy()).unwrap().to_string());
                                }
                                else { ui.label("?"); }
                                let _ = ui.button(&file.name);
                            });

                            row.col(|ui| {
                                ui.label(format_size(file.size));
                            });
                        });
                    }
                });



            if let Some(path) = new_path {
                self.current_path = path.clone();
                self.update_directory_list(&path);
            }
        });
    }
}