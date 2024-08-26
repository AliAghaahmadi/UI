use eframe::egui;
use egui_extras::TableBuilder;
use regex::Regex;

pub struct WifiNetwork {
    address: String,
    channel: String,
    frequency: String,
    quality: String,
    signal_level: String,
    encryption_key: String,
    essid: String,
    bit_rates: String,
    mode: String,
    extra: String,
}

pub fn parse_wifi_scan_output(output: &str) -> Vec<WifiNetwork> {
    let mut networks = Vec::new();
    let cells = output.split("Cell").skip(1);

    for cell in cells {
        let address_re = Regex::new(r"Address:\s+([\w:]+)").unwrap();
        let channel_re = Regex::new(r"Channel:(\d+)").unwrap();
        let frequency_re = Regex::new(r"Frequency:([\d.]+ GHz)").unwrap();
        let quality_re = Regex::new(r"Quality=(\d+)/(\d+)").unwrap();
        let signal_level_re = Regex::new(r"Signal level=(-?\d+) dBm").unwrap();
        let encryption_key_re = Regex::new(r"Encryption key:(\w+)").unwrap();
        let essid_re = Regex::new(r#"ESSID:"([^"]+)""#).unwrap();
        let bit_rates_re = Regex::new(r"Bit Rates:([\d\sMb/s;]+)").unwrap();
        let mode_re = Regex::new(r"Mode:(\w+)").unwrap();
        let extra_re = Regex::new(r"Extra:([^\n]+)").unwrap();

        let default = "Not found".to_string();

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

        let bit_rates = bit_rates_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        let mode = mode_re.captures(cell)
            .map(|caps| caps.get(1).map_or(default.clone(), |m| m.as_str().to_string()))
            .unwrap_or(default.clone());

        // Collect all extra fields into a single string
        let extra = extra_re.captures_iter(cell)
            .map(|caps| caps.get(1).map_or("", |m| m.as_str()))
            .collect::<Vec<&str>>()
            .join(", ");

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

// Function to display WiFi networks using egui
pub fn display_wifi_networks(ui: &mut egui::Ui, networks: &[WifiNetwork]) {
    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(0.0);

    table
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .header(20.0, |mut header| {
            header.col(|ui| { ui.strong("ESSID"); });
            header.col(|ui| { ui.strong("BSSID"); });
            header.col(|ui| { ui.strong("Channel"); });
            header.col(|ui| { ui.strong("Signal Level"); });
            header.col(|ui| { ui.strong("Frequency"); });
        })
        .body(|mut body| {
            for network in networks {
                body.row(20.0, |mut row| {
                    row.col(|ui| { ui.strong(&network.essid); });
                    row.col(|ui| { ui.label(&network.address); });
                    row.col(|ui| { ui.label(&network.channel); });
                    row.col(|ui| {
                        if let Ok(signal_level) = network.signal_level.parse::<f32>() {
                            let result = 100.0 + signal_level;
                            ui.label(format!("{}", result));
                        } else {
                            ui.label("Invalid signal level");
                        }
                    });
                    row.col(|ui| { ui.label(&network.frequency); });
                });
            }
        });
}
