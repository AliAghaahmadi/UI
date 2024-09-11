use std::fs::{self, File};
use std::io::{BufWriter};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use hound::WavReader;
use id3::{Tag, TagLike};
use csv::WriterBuilder;
use mp4ameta::Tag as Mp4Tag;
use eframe::egui;
use egui::Align;
use egui_extras::TableBuilder;

struct Audio {
    path: String,
    audio_type: String,
    title: String,
    artist: String,
    album: String,
    year: String,
    duration: String,
    bitrate: String,
    sample_rate: String,
    channels: String,
    bits_per_sample: String,
}

struct AudioPlayer {
    audio_list: Vec<Audio>,
}

impl AudioPlayer {
    fn new() -> Self {
        Self {
            audio_list: Vec::new(),
        }
    }

    fn update_audio_list(&mut self) {
        let home_dir = dirs::home_dir().expect("Unable to find home directory");
        let start_time = Instant::now();

        let count = Arc::new(AtomicU64::new(0));
        let csv_writer = Arc::new(Mutex::new(
            WriterBuilder::new()
                .delimiter(b',')
                .quote_style(csv::QuoteStyle::Always)
                .from_writer(BufWriter::new(File::create("../../../audio_files.csv").expect("Failed to create file")))
        ));
        let audio_list = Arc::new(Mutex::new(Vec::new()));

        {
            let mut writer = csv_writer.lock().expect("Failed to acquire lock");
            writer.write_record(&[
                "File Path",
                "Type",
                "Title",
                "Artist",
                "Album",
                "Year",
                "Duration",
                "Bitrate",
                "Sample Rate",
                "Channels",
                "Bits per Sample"
            ]).expect("Failed to write CSV header");
        }

        if let Err(e) = find_audio_files(&home_dir, Arc::clone(&csv_writer), Arc::clone(&count), Arc::clone(&audio_list)) {
            eprintln!("Error processing files: {}", e);
        }

        let duration = start_time.elapsed();
        println!("Found {} audio files in {:.2?}", count.load(Ordering::Relaxed), duration);

        let mut audio_list = audio_list.lock().expect("Failed to acquire lock");
        self.audio_list.append(&mut audio_list);

        println!("Results written to: audio_files.csv");
    }
}

fn is_audio_file(entry: &fs::DirEntry) -> bool {
    if let Some(extension) = entry.path().extension() {
        match extension.to_str().unwrap_or("").to_lowercase().as_str() {
            "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" => true,
            _ => false,
        }
    } else {
        false
    }
}

fn get_audio_details(path: &Path) -> Option<Audio> {
    let path_str = path.to_string_lossy().to_string();
    let file_name = path.file_stem()?.to_string_lossy().to_string();  // Get the file name without extension
    let extension = path.extension()?.to_str()?.to_lowercase();

    match extension.as_str() {
        "wav" => {
            match WavReader::open(path) {
                Ok(reader) => {
                    let spec = reader.spec();
                    let duration = reader.duration() as f64 / spec.sample_rate as f64;
                    Some(Audio {
                        path: path_str,
                        audio_type: "WAV".to_string(),
                        title: file_name.clone(),  // Use file name if title is not available
                        artist: "N/A".to_string(),
                        album: "N/A".to_string(),
                        year: "N/A".to_string(),
                        duration: format!("{:.2} seconds", duration),
                        bitrate: "N/A".to_string(),
                        sample_rate: format!("{}", spec.sample_rate),
                        channels: format!("{}", spec.channels),
                        bits_per_sample: format!("{}", spec.bits_per_sample),
                    })
                }
                Err(_e) => {
                    None
                }
            }
        },
        "mp3" => {
            match Tag::read_from_path(path) {
                Ok(tag) => {
                    Some(Audio {
                        path: path_str,
                        audio_type: "MP3".to_string(),
                        title: tag.title().unwrap_or(&file_name).to_string(),  // Use file name if title is not available
                        artist: tag.artist().unwrap_or("Unknown").to_string(),
                        album: tag.album().unwrap_or("Unknown").to_string(),
                        year: tag.year().map_or("Unknown".to_string(), |y| y.to_string()),
                        duration: "N/A".to_string(),
                        bitrate: "N/A".to_string(),
                        sample_rate: "N/A".to_string(),
                        channels: "N/A".to_string(),
                        bits_per_sample: "N/A".to_string(),
                    })
                }
                Err(_e) => {
                    None
                }
            }
        },
        "m4a" => {
            match Mp4Tag::read_from_path(path) {
                Ok(tag) => {
                    Some(Audio {
                        path: path_str,
                        audio_type: "M4A".to_string(),
                        title: tag.title().unwrap_or(&file_name).to_string(),  // Use file name if title is not available
                        artist: tag.artist().unwrap_or("Unknown").to_string(),
                        album: tag.album().unwrap_or("Unknown").to_string(),
                        year: tag.year().map_or("Unknown".to_string(), |y| y.to_string()),
                        duration: tag.duration().map_or("Unknown".to_string(), |d| format!("{:.2} seconds", d.as_secs_f64())),
                        bitrate: tag.avg_bitrate().map_or("Unknown".to_string(), |b| format!("{} kbps", b / 1000)),
                        sample_rate: "N/A".to_string(),
                        channels: "N/A".to_string(),
                        bits_per_sample: "N/A".to_string(),
                    })
                }
                Err(e) => {
                    eprintln!("Failed to read M4A file {}: {}", path_str, e);
                    None
                }
            }
        },
        _ => None
    }
}

fn find_audio_files(dir: &Path, csv_writer: Arc<Mutex<csv::Writer<BufWriter<File>>>>, count: Arc<AtomicU64>, audio_list: Arc<Mutex<Vec<Audio>>>) -> std::io::Result<()> {
    if dir.is_dir() {
        let entries: Vec<_> = fs::read_dir(dir)?.collect();

        entries.par_iter().for_each(|entry| {
            let entry = entry.as_ref().expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) = find_audio_files(&path, Arc::clone(&csv_writer), Arc::clone(&count), Arc::clone(&audio_list)) {
                    eprintln!("Failed to process subdirectory: {}", e);
                }
            } else if is_audio_file(&entry) {
                if let Some(details) = get_audio_details(&path) {
                    let mut writer = csv_writer.lock().expect("Failed to acquire lock");
                    if let Err(e) = writer.write_record(&[
                        &details.path,
                        &details.audio_type,
                        &details.title,
                        &details.artist,
                        &details.album,
                        &details.year,
                        &details.duration,
                        &details.bitrate,
                        &details.sample_rate,
                        &details.channels,
                        &details.bits_per_sample,
                    ]) {
                        eprintln!("Failed to write record for {}: {}", path.display(), e);
                    }
                    count.fetch_add(1, Ordering::Relaxed);
                    let mut list = audio_list.lock().expect("Failed to acquire lock");
                    list.push(details);
                }
            }
        });
    }
    Ok(())
}

impl eframe::App for AudioPlayer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load Audio Files").clicked() {
                self.update_audio_list();
            }

            if self.audio_list.is_empty() {
                ui.label("No audio files loaded.");
            } else {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(Align::Center))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .column(egui_extras::Column::initial(0.0).at_least(0.0))
                    .min_scrolled_height(0.0)
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.strong("Type"); });
                        header.col(|ui| { ui.strong("Title"); });
                        header.col(|ui| { ui.strong("Artist"); });
                        header.col(|ui| { ui.strong("Album"); });
                        header.col(|ui| { ui.strong("Year"); });
                        header.col(|ui| { ui.strong("Duration"); });
                        header.col(|ui| { ui.strong("Bitrate"); });
                        header.col(|ui| { ui.strong("Sample Rate"); });
                        header.col(|ui| { ui.strong("Channels"); });
                        header.col(|ui| { ui.strong("Bits/Per Sample"); });
                    })
                    .body(|mut body| {
                        for audio in &self.audio_list {
                            body.row(20.0, |mut row| {
                                row.col(|ui| { ui.label(&audio.audio_type); });
                                row.col(|ui| { ui.label(&audio.title); });
                                row.col(|ui| { ui.label(&audio.artist); });
                                row.col(|ui| { ui.label(&audio.album); });
                                row.col(|ui| { ui.label(&audio.year); });
                                row.col(|ui| { ui.label(&audio.duration); });
                                row.col(|ui| { ui.label(&audio.bitrate); });
                                row.col(|ui| { ui.label(&audio.sample_rate); });
                                row.col(|ui| { ui.label(&audio.channels); });
                                row.col(|ui| { ui.label(&audio.bits_per_sample); });
                            });
                        }
                    });
            }
        });
    }
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350.0, 450.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Custom Keypad App",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<AudioPlayer>::new(AudioPlayer::new()))
        }),
    )
}
