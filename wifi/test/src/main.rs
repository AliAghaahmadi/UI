mod wifi_scanner;

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::wifi_scanner::{Network, scan_networks, show_network_table};

#[derive(Default)]
pub struct MyApp {
    networks: Vec<Network>,
    scanning: bool, // Flag to indicate scanning status
    scan_result: Arc<Mutex<Option<Vec<Network>>>>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Custom Keyboard")
            .default_pos([100.0, 100.0])
            .title_bar(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    use egui::special_emojis::WIFI;
                    if ui.button(format!("{WIFI} egui on GitHub")).clicked() {
                        self.scanning = true; // Set the scanning flag
                        ctx.request_repaint(); // Request a repaint to update the UI immediately

                        let scan_result = self.scan_result.clone();

                        // Perform network scanning in another thread
                        thread::spawn(move || {
                            let networks = scan_networks();
                            let mut guard = scan_result.lock().unwrap();
                            *guard = Some(networks);
                        });
                    }

                    // Conditionally show the spinner next to the button
                    if self.scanning {
                        ui.add(egui::Spinner::new());
                        ui.label("Scanning...");
                    }
            });

            ui.label("Available Networks:");

            if let Some(networks) = self.scan_result.lock().unwrap().as_ref() {
                self.networks = networks.clone();
                self.scanning = false; // Reset the scanning flag
                ctx.request_repaint();
                show_network_table(ui, &self.networks);
            } else if !self.scanning {
                show_network_table(ui, &self.networks);
            }
        });
    }
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with RUST_LOG=debug).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Network Scanner",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<MyApp>::default())
        }),
    )
}
