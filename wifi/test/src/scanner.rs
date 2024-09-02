use eframe::egui;
use egui_extras::TableBuilder;
use regex::Regex;
use eframe::egui::{popup_below_widget, vec2, Button, Id, PopupCloseBehavior};

pub struct WifiNetwork {
    pub address: String,
    pub channel: String,
    pub frequency: String,
    pub quality: String,
    pub signal_level: String,
    pub encryption_key: String,
    pub essid: String,
    pub bit_rates: String,
    pub mode: String,
    pub extra: String,
}

pub fn parse_wifi_scan_output(output: &str) -> Vec<WifiNetwork> {
    let mut networks = Vec::new();
    let cells = output.split("Cell").skip(1);

    // Regular expressions to capture details
    let address_re = Regex::new(r"Address:\s+([\w:]+)").unwrap();
    let channel_re = Regex::new(r"Channel:(\d+)").unwrap();
    let frequency_re = Regex::new(r"Frequency:([\d.]+ GHz)").unwrap();
    let quality_re = Regex::new(r"Quality=(\d+)/(\d+)").unwrap();
    let signal_level_re = Regex::new(r"Signal level=(-?\d+) dBm").unwrap();
    let encryption_key_re = Regex::new(r"Encryption key:(\w+)").unwrap();
    let essid_re = Regex::new(r#"ESSID:"([^"]+)""#).unwrap();

    // Regex to capture the bit rates section
    let bit_rates_re = Regex::new(r"Bit Rates:([\s\S]*?)(\n[A-Z]|$)").unwrap();

    // Regex to capture the extra section
    let extra_re = Regex::new(r"Extra:([\s\S]*?)(\n[A-Z]|$)").unwrap();

    let mode_re = Regex::new(r"Mode:(\w+)").unwrap();

    let default = "Not found".to_string();

    for cell in cells {
        let address = address_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        let channel = channel_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        let frequency = frequency_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        let quality = quality_re.captures(cell)
            .map(|caps| format!("{}/{}", &caps[1], &caps[2]))
            .unwrap_or(default.clone());

        let signal_level = signal_level_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        let encryption_key = encryption_key_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        let essid = essid_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        // Capture bit rates and handle multiline content
        let bit_rates = bit_rates_re.captures(cell)
            .map(|caps| {
                let bit_rates_str = caps.get(1).map_or("", |m| m.as_str().trim());
                // Remove newlines and extra spaces
                bit_rates_str
                    .replace("\n", " ")
                    .replace("  ", " ")
                    .trim()
                    .to_string()
            })
            .unwrap_or(default.clone());

        // Capture extra and handle multiline content
        let extra = extra_re.captures(cell)
            .map(|caps| {
                let extra_str = caps.get(1).map_or("", |m| m.as_str().trim());
                // Clean up the captured extra field
                extra_str
                    .replace("\n", " ")
                    .replace("  ", " ")
                    .trim()
                    .to_string()
            })
            .unwrap_or(default.clone());

        let mode = mode_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        networks.push(WifiNetwork {
            address,
            channel,
            frequency,
            quality,
            signal_level,
            encryption_key,
            essid,
            bit_rates,
            mode: mode.clone(),
            extra: extra.clone(),
        });
    }

    networks
}


// Function to display WiFi networks using egui and return if the table is not empty
pub fn display_wifi_networks(ui: &mut egui::Ui, networks: &[WifiNetwork]) -> bool {
    if networks.is_empty() {
        return false;
    }

    let table = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(0.0);

    table
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .header(20.0, |mut header| {
            header.col(|ui| { ui.strong("ESSID"); });
            header.col(|ui| { ui.strong("BSSID"); });
            header.col(|ui| { ui.strong("Signal Level"); });
        })

        .body(|mut body| {
            for network in networks {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        let response =  ui.button(check_name(&network.essid));

                        let popup_id = Id::new(format!("popup_id {}", network.essid));

                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }

                        popup_below_widget(
                            ui,
                            popup_id,
                            &response,
                            PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.horizontal(|ui|
                                    {
                                        if ui.add_sized(vec2(24.0, 24.0), Button::new("❌")).clicked() { ui.memory_mut(|mem| mem.close_popup()); };
                                    });

                                egui::ScrollArea::vertical().show(ui, |ui|
                                    {
                                        ui.set_max_width(300.0);
                                        ui.label(format!("Frequency: {}", network.frequency));
                                        ui.separator();
                                        ui.label(format!("Encryption Key: {}", normalize_extra_text(&*network.encryption_key)));
                                        ui.separator();
                                        ui.label(format!("Channel: {}", normalize_extra_text(&*network.channel)));
                                        ui.separator();
                                        ui.label(format!("Bit Rates: {}", normalize_extra_text(&*network.bit_rates)));
                                        ui.separator();
                                        ui.label(format!("Extra: {}", normalize_extra_text(&*network.extra)));
                                    })
                            },
                        );
                    });
                    row.col(|ui| { ui.label(&network.address); });
                    row.col(|ui| { ui.label(show_quality(&network.quality)); });
                });
            }
        });

    true
}

fn normalize_extra_text(extra: &str) -> String {
    let binding = extra
        .replace('\n', " ")
        .replace("\r", "")
        .replace("  ", " ");
    let normalized = binding
        .trim();

    let mut result = normalized.to_string();
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    result
}

fn show_quality(input: &str) -> String {
    let parts: Vec<&str> = input.split('/').collect();
    if let Some(first_part) = parts.get(0) {
        if let Ok(value) = first_part.trim().parse::<f32>() {
            if value > 52.5 { "📶 Excellent".to_string() }
            else if value > 35.0 { "📶 Good".to_string() }
            else if value > 17.5 { "📶 Fair".to_string() }
            else if value > 0.0 { "📶 Poor".to_string() }
            else { "📶 No Signal".to_string() }
        } else {
            "📶 Invalid Input".to_string()
        }
    } else {
        "📶 No Signal".to_string()
    }
}

fn check_name(name: &str) -> String
{
    if name.contains(&"x00".to_string()){
        "Hidden".to_string()
    } else {
        name.to_string()
    }
}