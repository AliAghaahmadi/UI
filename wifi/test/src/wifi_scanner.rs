#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
use std::process::Command;
use std::str;
use regex::Regex;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

#[derive(Debug, Clone)]
pub struct Network {
    pub bssid: String,
    pub ssid: String,
    pub channel: String,
    pub signal: String,
    pub bars: String,
    pub security: String,
}

pub fn scan_networks() -> Vec<Network> {
    let output = Command::new("nmcli")
        .args(&["-f", "BSSID,SSID,CHAN,SIGNAL,BARS,SECURITY", "device", "wifi", "list"])
        .output()
        .expect("Failed to execute nmcli");

    let output_str = str::from_utf8(&output.stdout).expect("Invalid UTF-8");
    println!("nmcli output:\n{}", output_str); // Debugging: Print the nmcli output
    parse_networks(output_str)
}

fn parse_networks(output: &str) -> Vec<Network> {
    let mut networks = Vec::new();

    // Regular expression to capture the columns with variable spacing
    let re = Regex::new(r"(?P<bssid>\S+)\s+(?P<ssid>\S+)\s+(?P<channel>\d+)\s+(?P<signal>-?\d+)\s+(?P<bars>\S+)\s+(?P<security>.+)").unwrap();

    // Iterate over each line, skipping the header
    for line in output.lines().skip(1) {
        if let Some(caps) = re.captures(line) {
            let network = Network {
                bssid: caps.name("bssid").unwrap().as_str().to_string(),
                ssid: caps.name("ssid").unwrap().as_str().to_string(),
                channel: caps.name("channel").unwrap().as_str().to_string(),
                signal: caps.name("signal").unwrap().as_str().to_string(),
                bars: caps.name("bars").unwrap().as_str().to_string(),
                security: caps.name("security").unwrap().as_str().to_string(),
            };
            networks.push(network);
        }
    }

    networks
}

pub fn show_network_table(ui: &mut egui::Ui, networks: &[Network]) {
    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(150.0).at_least(25.0))
        .column(Column::initial(150.0).at_least(25.0))
        .column(Column::initial(60.0).at_least(25.0))
        .min_scrolled_height(0.0);

    table.header(20.0, |mut header| {
        header.col(|ui| { ui.strong("BSSID"); });
        header.col(|ui| { ui.strong("SSID"); });
        header.col(|ui| { ui.strong("Channel"); });
    })
        .body(|mut body| {
            for network in networks {
                body.row(20.0, |mut row| {
                    row.col(|ui| { ui.label(&network.ssid); });
                    row.col(|ui| { ui.label(&network.bars); });
                    row.col(|ui| { ui.label(&network.security); });
                });
            }
        });
}

