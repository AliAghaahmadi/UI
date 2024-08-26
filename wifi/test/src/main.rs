mod scanner;

use eframe::egui;
use std::process::Command;
use scanner::{WifiNetwork, parse_wifi_scan_output, display_wifi_networks};

pub struct WifiScannerApp {
    pub wifi_networks: Vec<WifiNetwork>,
    pub scan_requested: bool,
}

impl Default for WifiScannerApp {
    fn default() -> Self {
        Self {
            wifi_networks: Vec::new(),
            scan_requested: false,
        }
    }
}

impl WifiScannerApp {
    pub fn scan_wifi_networks(&mut self) {
        let wifi_adapter = "wlp3s0";

        let output = Command::new("./wifi/test/src/sudo_wrapper.sh")
            .arg("iwlist")
            .arg(wifi_adapter)
            .arg("scan")
            .output()
            .expect("Failed to execute command");

        let output_str = String::from_utf8_lossy(&output.stdout);

        self.wifi_networks = parse_wifi_scan_output(&output_str);
        self.scan_requested = true;
    }

    fn display_wifi_table(&self, ui: &mut egui::Ui) {
        if self.scan_requested {
            display_wifi_networks(ui, &self.wifi_networks);
        } else {
            ui.label("Click 'Scan WiFi Networks' to see the results.");
        }
    }
}

impl eframe::App for WifiScannerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WiFi Scanner");

            if ui.button("Scan WiFi Networks").clicked() {
                self.scan_wifi_networks();
            }

            self.display_wifi_table(ui);
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
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