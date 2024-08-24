use std::collections::HashMap;
use eframe::egui;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui::{popup_above_or_below_widget, popup_below_widget, AboveOrBelow, Id, PopupCloseBehavior};
use eframe::egui::TextWrapMode::Wrap;

// Define the Folder struct
#[derive(Debug, Clone)]
pub struct Folder {
    pub dir: String,
    pub name: String,
    pub size: Arc<Mutex<Option<u64>>>,
    pub calculating: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct File {
    pub dir: String,
    pub name: String,
    pub size: Option<u64>,
}

// Implement Default for Folder
impl Default for Folder {
    fn default() -> Self {
        Self {
            dir: String::new(),
            name: String::new(),
            size: Arc::new(Mutex::new(None)),
            calculating: Arc::new(Mutex::new(false)),
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

// Define the FileBrowserApp struct
pub struct FileBrowserApp {
    pub current_path: String,
    pub files: Vec<File>,
    pub directories: Vec<Folder>,
    pub selected: File,
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
        };
        app.update_directory_list(&start_path);
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
                                let response = ui.button(&file.name);

                                let popup_id = Id::new(&file.name);

                                if response.secondary_clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }

                                popup_above_or_below_widget(
                                    ui,
                                    popup_id,
                                    &response,
                                    AboveOrBelow::Above,
                                    PopupCloseBehavior::IgnoreClicks,
                                    |ui| {
                                        ui.set_min_width(100.0);

                                        let _ = ui.button("Copy");
                                        let _ = ui.button("Cut");
                                        let _ = ui.button("Rename");
                                        if ui.button("Delete").clicked()
                                        {
                                            ui.memory_mut(|mem| mem.close_popup());
                                        }
                                    },
                                );
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

fn format_size(size: Option<u64>) -> String {
    match size {
        Some(size) => {
            let units = ["B", "KB", "MB", "GB", "TB"];
            let mut size = size as f64;
            let mut unit = 0;
            while size >= 1024.0 && unit < units.len() - 1 {
                size /= 1024.0;
                unit += 1;
            }
            format!("{:.2} {}", size, units[unit])
        }
        None => "Unknown".to_string(),
    }
}

fn directory_size(folder: &Folder) {
    let dir_path = folder.dir.clone();
    let size = Arc::clone(&folder.size);
    let calculating = Arc::clone(&folder.calculating);

    thread::spawn(move || {
        *calculating.lock().unwrap() = true;
        let calculated_size = dir_size(Path::new(&dir_path)).ok();
        *size.lock().unwrap() = calculated_size;
        *calculating.lock().unwrap() = false;
    });

    fn dir_size(path: &Path) -> Result<u64, std::io::Error> {
        let mut size = 0;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    size += dir_size(&path)?;
                } else {
                    size += metadata(path)?.len();
                }
            }
        }
        Ok(size)
    }
}

fn extension_icon (s: &str) -> Result<String, std::io::Error> {
    let dictionary: HashMap<&str, &str> = HashMap::from([
        ("png", "🖼"),
        ("jpg", "🖼"),
        ("jpeg", "🖼"),
        ("txt", "🗒"),
        ("toml", "📔"),
        ("lock", "📔"),
        ("rs", "®"),
    ]);

    let default_value = "?";

    if let Some(value) = dictionary.get(s) {
        Ok(value.to_string())
    } else {
        Ok(default_value.to_string())
    }
}