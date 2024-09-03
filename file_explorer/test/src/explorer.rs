use eframe::egui;
use rayon::prelude::*;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use eframe::egui::{popup_above_or_below_widget, AboveOrBelow, Id, PopupCloseBehavior};
use eframe::epaint::Color32;
use egui::{vec2, RichText, TextEdit};

#[derive(Debug, Clone)]
pub struct Folder {
    pub dir: String,
    pub name: String,
    pub size: Arc<Mutex<Option<u64>>>,
    pub calculating: Arc<Mutex<bool>>,
    pub error: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
pub struct File {
    pub dir: String,
    pub name: String,
    pub size: Option<u64>,
}

impl Default for Folder {
    fn default() -> Self {
        Self {
            dir: String::new(),
            name: String::new(),
            size: Arc::new(Mutex::new(None)),
            calculating: Arc::new(Mutex::new(false)),
            error: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for File {
    fn default() -> Self {
        Self {
            dir: String::new(),
            name: String::new(),
            size: None,
        }
    }
}

pub struct FileBrowserApp {
    pub current_path: String,
    pub files: Vec<File>,
    pub directories: Vec<Folder>,
    pub selected: File,
    pub search: String,
    pub previous_search: String,
}

impl Default for FileBrowserApp {
    fn default() -> Self {
        let start_path = if cfg!(target_os = "windows") {
            "C:\\".to_string()
        } else {
            "/".to_string()
        };

        let mut app = Self {
            current_path: start_path.clone(),
            files: Vec::new(),
            directories: Vec::new(),
            selected: File::default(),
            search: String::new(),
            previous_search: String::new(),
        };
        app.update_directory_list(&start_path);
        app
    }
}

fn search_in_directory_parallel(dir: &Path, search_term: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        let entries: Vec<_> = entries.filter_map(Result::ok).collect();

        let matched_paths: Vec<_> = entries
            .par_iter()
            .flat_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().and_then(|n| n.to_str()).unwrap_or("").contains(search_term) {
                        vec![path]
                    } else {
                        search_in_directory_parallel(&path, search_term)
                    }
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains(search_term) {
                        vec![path]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            })
            .collect();

        results.extend(matched_paths);
    }

    results
}

impl FileBrowserApp {
    fn update_directory_list(&mut self, path: &str) {
        self.files.clear();
        self.directories.clear();

        let (tx, rx) = mpsc::channel();
        let dirpath = Path::new(path).to_owned();
        let search_term = self.search.clone();

        thread::spawn(move || {
            let paths = search_in_directory_parallel(&dirpath, &search_term);
            tx.send(paths).expect("Failed to send data through channel");
        });

        // In the main thread, receive the results and update the UI
        let paths = rx.recv().expect("Failed to receive data through channel");

        for path in paths {
            let name = path.file_name().unwrap().to_string_lossy().to_string();

            if path.is_dir() {
                let dir_path = path.to_string_lossy().to_string();
                let folder = Folder {
                    dir: dir_path,
                    name,
                    size: Arc::new(Mutex::new(None)),
                    calculating: Arc::new(Mutex::new(false)),
                    error: Arc::new(Mutex::new(None)),
                };

                self.directories.push(folder);
            } else {
                let file = File {
                    dir: path.to_string_lossy().to_string(),
                    name,
                    size: metadata(&path).ok().map(|m| m.len()),
                };
                self.files.push(file);
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

            // Navigation buttons and search
            ui.horizontal(|ui| {
                if ui.button("⏴").clicked() {
                    if let Some(parent) = Path::new(&self.current_path).parent() {
                        if let Some(parent_str) = parent.to_str() {
                            let parent_path = parent_str.to_string();
                            self.current_path = parent_path.clone();
                            self.search = "".to_string();
                            self.update_directory_list(&parent_path);
                        }
                    }
                }

                ui.horizontal(|ui| {
                    for parent in get_parent_directories(Path::new(&self.current_path)) {
                        if parent.file_name().is_some() {
                            if ui.button(parent.file_name().unwrap().to_string_lossy()).clicked() {
                                self.current_path = parent.to_string_lossy().parse().unwrap();
                                self.search = "".to_string();
                                self.update_directory_list(&self.current_path.clone());
                            }

                            ui.label("/");
                        }
                    }

                    let text = ui.add(TextEdit::singleline(&mut (self.search)).desired_width(50.0));
                    text.request_focus();

                    if self.previous_search != self.search {
                        if ui.button("🔎").clicked() {
                            self.update_directory_list(&self.current_path.clone());
                            self.previous_search = self.search.clone();
                        }
                    }
                });
            });

            ui.separator();

            let mut new_path = None;

            let combined_table = egui_extras::TableBuilder::new(ui)
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
                    for directory in &mut self.directories {
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
                                                    directory_size(directory);
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
                                                    ui.label(format_size(size));
                                                });
                                            }
                                        })
                                    },
                                );
                            });
                        });
                    }

                    for file in &self.files {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                let path = Path::new(&file.name);
                                if path.extension() != None
                                { ui.label(extension_icon(path.extension().unwrap().to_str().unwrap()).unwrap().to_string()); }
                                else { ui.label("❓"); }
                                let file_btn = ui.button(&file.name);

                                if file_btn.clicked() {
                                    new_path = Some(file.dir.clone());
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
                                            ui.label(format!("Size: {}", format_size(Some(size))));
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
                self.current_path = path;
                self.search = "".to_string();
                self.update_directory_list(&self.current_path.clone());
            }
        });
    }
}

fn directory_size(folder: &Folder) {
    let folder_path = folder.dir.clone();
    let calculating = folder.calculating.clone();
    let error = folder.error.clone();
    let size = folder.size.clone();

    thread::spawn(move || {
        *calculating.lock().unwrap() = true;
        *error.lock().unwrap() = None;

        let result = calculate_size(&folder_path);

        *calculating.lock().unwrap() = false;
        match result {
            Ok(s) => *size.lock().unwrap() = Some(s),
            Err(e) => *error.lock().unwrap() = Some(e),
        }
    });
}

fn calculate_size(path: &str) -> Result<u64, String> {
    let mut total_size = 0;

    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            total_size += calculate_size(path.to_str().ok_or("Invalid path")?)?;
        } else {
            total_size += metadata(&path).map_err(|e| e.to_string())?.len();
        }
    }

    Ok(total_size)
}

fn format_size(size: Option<u64>) -> String {
    match size {
        Some(bytes) => {
            if bytes < 1024 {
                format!("{} B", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else if bytes < 1024 * 1024 * 1024 {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        }
        None => "Unknown".to_string(),
    }
}

fn extension_icon(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "txt" => Some("📄"),
        "jpg" | "jpeg" | "png" => Some("🖼"),
        "mp3" | "wav" => Some("🎵"),
        "pdf" => Some("📄"),
        "zip" => Some("📦"),
        _ => Some("❓"),
    }
}
