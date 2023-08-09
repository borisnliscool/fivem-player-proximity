use eframe::egui;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::{env, io::{Read, Seek, SeekFrom}};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{collections::HashMap, thread};

fn get_logs_folder() -> Result<String, env::VarError> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let subfolder_path = PathBuf::from(local_app_data).join("FiveM/FiveM.app/logs");
    Ok(subfolder_path.to_string_lossy().into_owned())
}

fn get_latest_log_path(logs_folder: &str) -> Option<PathBuf> {
    let log_files = fs::read_dir(logs_folder)
        .ok()?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            if metadata.is_file() {
                Some((entry.path(), metadata.modified().ok()?))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    log_files
        .iter()
        .max_by_key(|(_, time)| *time)
        .map(|(path, _)| path.to_owned())
}

struct Player {
    name: String,
}

lazy_static! {
    static ref NEARBY_PLAYERS: Mutex<HashMap<String, Player>> = Mutex::new(HashMap::new());
}

fn process_log_diff(last_contents: &str, new_contents: &str) {
    let diff = difference::Changeset::new(last_contents, new_contents, "\n");
    let enter_regex = Regex::new(r"Creating physical player \d+ \((?P<name>.+?)\)").unwrap();
    let exit_regex = Regex::new(r"Processing removal for player \d+ \((?P<name>.+?)\)").unwrap();

    for diff_item in diff.diffs {
        match diff_item {
            difference::Difference::Add(addition) => {
                for cap in enter_regex.captures_iter(&addition) {
                    let name = &cap["name"];

                    let mut players = NEARBY_PLAYERS.lock().unwrap();
                    players.insert(
                        name.to_string(),
                        Player {
                            name: name.to_string(),
                        },
                    );
                }

                for cap in exit_regex.captures_iter(&addition) {
                    let name = &cap["name"];

                    let mut players = NEARBY_PLAYERS.lock().unwrap();
                    players.remove(name);
                };
            }
            _ => {}
        }
    }
}

struct MyApp {}

impl Default for MyApp {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let players = NEARBY_PLAYERS.lock().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Nearby Players ({})", players.len()));
            // Iterate over player names and print them
            ui.vertical(|ui| {
                for player in players.values() {
                    ui.label(player.name.clone());
                }
            });

            ctx.request_repaint();
        });
    }
}

fn run_ui() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "FiveM nearby player detector",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    );
}

fn watch_log_file(log_file: &PathBuf, mut last_file_size: u64) -> Result<(), failure::Error> {
    let mut watcher = Hotwatch::new()?;
    watcher.watch(log_file, move |event: Event| {
        if let Ok(metadata) = fs::metadata(&event.paths[0]) {
            let new_file_size = metadata.len();
            if new_file_size > last_file_size {
                let file = fs::File::open(&event.paths[0]);
                if let Ok(mut file) = file {
                    if file.seek(SeekFrom::Start(last_file_size)).is_ok() {
                        let mut new_contents = String::new();
                        if file.read_to_string(&mut new_contents).is_ok() {
                            process_log_diff("", &new_contents); // Compare with empty string for initial case
                            last_file_size = new_file_size;
                        }
                    }
                }
            }
        }

        Flow::Continue
    })?;

    println!("Watching logs file");
    watcher.run();

    Ok(())
}


fn run_watcher() -> Result<(), failure::Error> {
    let logs_folder = get_logs_folder()?;
    let latest_log_file = get_latest_log_path(&logs_folder).expect("Couldn't find log file");

    let initial_file_size = fs::metadata(&latest_log_file)?.len();
    watch_log_file(&latest_log_file, initial_file_size)?;

    Ok(())
}

fn main() {
    // Spawn the file watcher thread
    let file_watcher = thread::spawn(move || {
        if let Err(err) = run_watcher() {
            eprintln!("File watcher error: {}", err);
        }
    });

    // Run the UI on the main thread
    run_ui();

    // Wait for the file watcher thread to finish
    let _ = file_watcher.join().expect("File watcher thread panicked");
}
