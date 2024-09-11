use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui;
use eframe::egui::{vec2, Button};

mod scanner;

use scanner::{display_wifi_networks, parse_wifi_scan_output, WifiNetwork};

pub struct WifiScannerApp {
    wifi_networks: Arc<Mutex<Vec<WifiNetwork>>>,
    scanning: Arc<Mutex<bool>>,
    scan_error: Arc<Mutex<Option<String>>>,
}

impl Default for WifiScannerApp {
    fn default() -> Self {
        Self {
            wifi_networks: Arc::new(Mutex::new(Vec::new())),
            scanning: Arc::new(Mutex::new(false)),
            scan_error: Arc::new(Mutex::new(None)),
        }
    }
}

impl WifiScannerApp {
    pub fn scan_wifi_networks(&self) {
        let wifi_networks = Arc::clone(&self.wifi_networks);
        let scanning = Arc::clone(&self.scanning);
        let scan_error = Arc::clone(&self.scan_error);

        let wifi_adapter = "wlp3s0".to_string();

        thread::spawn(move || {
            *scanning.lock().unwrap() = true;
            match Command::new("./wifi/test/src/sudo_wrapper.sh")
                .arg("iwlist")
                .arg(&wifi_adapter)
                .arg("scan")
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let networks = parse_wifi_scan_output(&output_str);

                    let mut wifi_networks = wifi_networks.lock().unwrap();
                    *wifi_networks = networks;
                    *scan_error.lock().unwrap() = None;
                }
                Err(e) => {
                    *scan_error.lock().unwrap() = Some(format!("Failed to execute scan command: {}", e));
                    let mut wifi_networks = wifi_networks.lock().unwrap();
                    *wifi_networks = Vec::new(); // Clear the list on failure
                }
            }
            *scanning.lock().unwrap() = false;
        });
    }

    fn display_wifi_table(&self, ui: &mut egui::Ui) {
        let wifi_networks = self.wifi_networks.lock().unwrap();
        display_wifi_networks(ui, &wifi_networks);

        if let Some(ref error) = *self.scan_error.lock().unwrap() {
            ui.label(egui::RichText::new(error).color(egui::Color32::RED));
        }
    }
}

impl eframe::App for WifiScannerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WiFi Scanner");

            if *self.scanning.lock().unwrap() {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label("Scanning...");
                });
            } else if ui.add_sized(vec2(50.0, 24.0), Button::new("🖧 Scan")).clicked() {
                self.scan_wifi_networks();
            }

            self.display_wifi_table(ui);
        });

        ctx.request_repaint(); // Ensure the UI is constantly refreshed
    }
}


fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([450.0, 450.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WiFi Scanner",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<WifiScannerApp>::default())
        }),
    )
}
