use crate::egui::Button;
use eframe::egui;
use rayon::prelude::*;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use egui::{Color32, Context, Style, TextEdit, Ui};
use crate::list::list_explorer;

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
    pub selected_option: Option<usize>,
    pub settings: bool,
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
            selected_option: None,
            settings: false,
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
    pub(crate) fn update_directory_list(&mut self, path: &str) {
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

    pub fn directory_size(folder: &Folder) {
        let folder_path = folder.dir.clone();
        let calculating = folder.calculating.clone();
        let error = folder.error.clone();
        let size = folder.size.clone();

        thread::spawn(move || {
            *calculating.lock().unwrap() = true;
            *error.lock().unwrap() = None;

            let result = Self::calculate_size(&folder_path);

            *calculating.lock().unwrap() = false;
            match result {
                Ok(s) => *size.lock().unwrap() = Some(s),
                Err(e) => *error.lock().unwrap() = Some(e),
            }
        });
    }

    pub fn calculate_size(path: &str) -> Result<u64, String> {
        let mut total_size = 0;

        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                total_size += Self::calculate_size(path.to_str().ok_or("Invalid path")?)?;
            } else {
                total_size += metadata(&path).map_err(|e| e.to_string())?.len();
            }
        }

        Ok(total_size)
    }

    pub fn format_size(size: Option<u64>) -> String {
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

    pub fn extension_icon(extension: &str) -> Option<&'static str> {
        match extension.to_lowercase().as_str() {
            "txt" => Some("📄"),
            "jpg" | "jpeg" | "png" => Some("🖼"),
            "mp3" | "wav" => Some("🎵"),
            "pdf" => Some("📄"),
            "zip" => Some("📦"),
            _ => Some("❓"),
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

fn save_style_to_file(ctx: &Context) -> std::io::Result<()> {
    let style_json = serde_json::to_string_pretty(&ctx.style()).expect("Failed to serialize style");
    fs::write("../example.json", style_json)
}

pub fn load_style_from_file(ctx: &Context) -> std::io::Result<()> {
    let style_json = fs::read_to_string("/home/ali/Projects/UI/example.json")?;
    let new_style: Style = serde_json::from_str(&style_json).expect("Failed to deserialize style");
    ctx.set_style(new_style);
    Ok(())
}

impl eframe::App for FileBrowserApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.settings {
            egui::Window::new("🔧 Settings")
                .vscroll(true)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                    if ui.button("Save").clicked() { save_style_to_file(ctx).expect("TODO: panic message"); }
                });
        }

        else { load_style_from_file(&*ctx).expect("TODO: panic message"); }

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

                    if self.current_path != "/"
                    {
                        let text = ui.add(TextEdit::singleline(&mut (self.search)).desired_width(50.0));

                        if self.previous_search != self.search {
                            if ui.button("🔎").clicked() || text.clicked_elsewhere() {
                                self.update_directory_list(&self.current_path.clone());
                                self.previous_search = self.search.clone();
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        /*if ui.radio(self.selected_option == Some(0), "Option 1").clicked() { self.selected_option = Some(0); }
                        if ui.radio(self.selected_option == Some(1), "Option 2").clicked() { self.selected_option = Some(1); }
                        if ui.radio(self.selected_option == Some(2), "Option 3").clicked() { self.selected_option = Some(2); }*/

                        toggle_button("Settings", &mut self.settings, ui);
                    });
                });
            });

            ui.separator();

            list_explorer(self, ui);
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