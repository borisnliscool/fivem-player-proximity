use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use regex::Regex;
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    thread,
};

use crate::{
    files::{get_latest_log_path, get_logs_folder},
    Player, NEARBY_PLAYERS,
};

fn process_log_diff(last_contents: &str, new_contents: &str) {
    let diff = difference::Changeset::new(last_contents, new_contents, "\n");
    let enter_regex = Regex::new(r"Creating physical player \d+ \((?P<name>.+?)\)").unwrap();
    let exit_regex = Regex::new(r"Processing removal for player \d+ \((?P<name>.+?)\)").unwrap();

    for diff_item in diff.diffs {
        match diff_item {
            difference::Difference::Add(addition) => {
                for cap in exit_regex.captures_iter(&addition) {
                    let name = &cap["name"];

                    let mut players = NEARBY_PLAYERS.lock().unwrap();
                    players.remove(name);
                }

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
            }
            _ => {}
        }
    }
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

pub fn start_file_watcher_thread() -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(err) = run_watcher() {
            eprintln!("File watcher error: {}", err);
        }
    })
}
