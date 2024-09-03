use crate::egui::Color32;
use eframe::egui;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use eframe::egui::{popup_above_or_below_widget, popup_below_widget, AboveOrBelow, Id, PopupCloseBehavior, RichText};

// Define Folder and File structs
#[derive(Debug, Clone, Default)]
pub struct Folder {
    pub dir: String,
    pub name: String,
    pub size: Arc<Mutex<Option<u64>>>,
    pub calculating: Arc<Mutex<bool>>,
    pub error: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Default)]
pub struct File {
    pub dir: String,
    pub name: String,
    pub size: Option<u64>,
}

// Define FileBrowserApp struct
pub struct FileBrowserApp {
    pub current_path: String,
    pub files: Vec<File>,
    pub directories: Vec<Folder>,
    pub selected: File,
    pub search: String,
}

impl Default for FileBrowserApp {
    fn default() -> Self {
        let start_path = if cfg!(target_os = "windows") { "C:\\" } else { "/" };
        let mut app = Self {
            current_path: start_path.to_string(),
            files: Vec::new(),
            directories: Vec::new(),
            selected: File::default(),
            search: String::new(),
        };
        app.update_directory_list(&app.current_path.clone());
        app
    }
}

impl FileBrowserApp {
    fn search_in_directory_parallel(&self, dir: &Path) -> Vec<PathBuf> {
        let mut results = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            let entries: Vec<_> = entries.filter_map(Result::ok).collect();

            let mut matched_paths: Vec<_> = entries
                .par_iter()
                .flat_map(|entry| {
                    let path = entry.path();
                    if path.is_dir() {
                        self.search_in_directory_parallel(&*path)
                    } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.contains(&self.search.clone()) {
                            vec![path]
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                })
                .collect();

            results.append(&mut matched_paths);
        }

        results
    }

    fn search_files_and_folders(&self, path: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let search_lower = self.search.to_lowercase();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();

                if search_lower.is_empty() || name.to_lowercase().contains(&search_lower) {
                    results.push(path);
                }
            }
        } else {
            eprintln!("Error reading directory: {}", path);
        }

        results
    }

    fn update_directory_list(&mut self, path: &str) {
        let dirpath = Path::new(path);
        if !dirpath.exists() || !dirpath.is_dir() {
            eprintln!("The specified path does not exist or is not a directory.");
            return;
        }

        let paths = self.search_in_directory_parallel(dirpath);

        self.files.clear();
        self.directories.clear();

        for path in paths {
            let name = path.file_name().unwrap().to_string_lossy().to_string();

            if path.is_dir() {
                self.directories.push(Folder {
                    dir: path.to_string_lossy().to_string(),
                    name,
                    size: Arc::new(Mutex::new(None)),
                    calculating: Arc::new(Mutex::new(false)),
                    error: Arc::new(Mutex::new(None)),
                });
            } else {
                self.files.push(File {
                    dir: path.to_string_lossy().to_string(),
                    name,
                    size: metadata(&path).ok().map(|m| m.len()),
                });
            }
        }
    }
}

pub fn delete_file(file_path: &str) {
    let path = file_path.strip_prefix('/').unwrap_or(file_path);
    if let Err(e) = fs::remove_file(path) {
        eprintln!("Error removing file: {}", e);
    }
}

fn get_parent_directories(path: &Path) -> Vec<PathBuf> {
    let mut parents = Vec::new();
    let mut current_path = path.to_path_buf();
    while let Some(parent) = current_path.parent() {
        parents.push(current_path.clone());
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
                    if let Some(parent) = Path::new(&self.current_path).parent().and_then(Path::to_str) {
                        self.current_path = parent.to_string();
                        self.search.clear();
                        self.update_directory_list(&self.current_path.clone());
                    }
                }

                ui.horizontal(|ui| {
                    for parent in get_parent_directories(Path::new(&self.current_path)) {
                        if let Some(file_name) = parent.file_name() {
                            if ui.button(file_name.to_string_lossy()).clicked() {
                                self.current_path = parent.to_string_lossy().to_string();
                                self.search.clear();
                                self.update_directory_list(&self.current_path.clone());
                            }
                            ui.label("/");
                        }
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = ui.button("🔎");
                    let popup_id = Id::new("Search");

                    if response.clicked() {
                        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                    }

                    popup_below_widget(ui, popup_id, &response, PopupCloseBehavior::CloseOnClickOutside, |ui| {
                        ui.horizontal(|ui| {
                            ui.strong("🔎: ");
                            ui.set_min_width(300.0);
                            if ui.text_edit_singleline(&mut self.search).changed() {
                                self.update_directory_list(&self.current_path.clone());
                            }
                        });
                    });
                });
            });

            ui.separator();

            let mut new_path = None;

            let combined_table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::initial(150.0).at_least(25.0))
                .column(egui_extras::Column::initial(150.0).at_least(25.0))
                .min_scrolled_height(0.0);

            combined_table
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.strong("Name"); });
                    header.col(|ui| { ui.strong("Size"); });
                })
                .body(|mut body| {
                    for folder in &mut self.directories {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("📁");
                                let dir_btn = ui.button(&folder.name);
                                if dir_btn.clicked() {
                                    new_path = Some(format!("{}/{}", self.current_path, folder.name));
                                }
                                let id = Id::new(format!("Del {}", folder.name));
                                if dir_btn.secondary_clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(id));
                                }
                                popup_above_or_below_widget(ui, id, &dir_btn, AboveOrBelow::Above, PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                    ui.set_min_width(100.0);
                                    let size = folder.size.lock().unwrap().clone();
                                    let calculating = *folder.calculating.lock().unwrap();
                                    let error = folder.error.lock().unwrap().clone();
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Name: ");
                                            ui.strong(&folder.name);
                                        });
                                        if let Some(err) = error {
                                            ui.horizontal(|ui| {
                                                ui.label("Error: ");
                                                ui.label(RichText::new("Something Went Wrong").color(Color32::RED));
                                            });
                                        } else if size.is_none() && calculating {
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
                                        ui.horizontal_centered(|ui| {
                                            ui.button("Rename");
                                            ui.button("Copy");
                                            ui.button("Paste");
                                            ui.button("Delete");
                                        });
                                    });
                                });
                            });
                        });
                    }

                    for file in &self.files {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                let ext = Path::new(&file.dir).extension().map(|e| e.to_string_lossy()).unwrap_or_else(|| "❓".to_string().into());
                                ui.label(extension_icon(&ext).unwrap_or("❓"));
                                ui.button(&file.name);
                            });
                            row.col(|ui| {
                                ui.label(file.size.map_or("Unknown".to_string(), |size| format_size(Some(size))));
                            });
                        });
                    }
                });

            if let Some(path) = new_path {
                self.current_path = path;
                self.search.clear();
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
        match calculate_size(&folder_path) {
            Ok(s) => *size.lock().unwrap() = Some(s),
            Err(e) => *error.lock().unwrap() = Some(e),
        }
        *calculating.lock().unwrap() = false;
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
        Some(s) => if s < 1024 {
            format!("{} B", s)
        } else if s < 1_048_576 {
            format!("{:.2} KB", s as f64 / 1024.0)
        } else if s < 1_073_741_824 {
            format!("{:.2} MB", s as f64 / 1_048_576.0)
        } else {
            format!("{:.2} GB", s as f64 / 1_073_741_824.0)
        },
        None => "Unknown".to_string(),
    }
}

fn extension_icon(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "txt" => Some("📝"),
        "jpg" | "jpeg" | "png" => Some("🖼️"),
        "pdf" => Some("📄"),
        "mp3" => Some("🎵"),
        "mp4" => Some("🎥"),
        "zip" | "rar" => Some("📦"),
        _ => None,
    }
}
