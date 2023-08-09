use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

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

fn print_all_players() {
    let players = NEARBY_PLAYERS.lock().unwrap();

    println!("All Players:");
    for (_, plr) in players.iter() {
        println!("{}", plr.name);
    }
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
                }

                print_all_players();
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let logs_folder = get_logs_folder()?;
    let latest_log_file = get_latest_log_path(&logs_folder).expect("Couldn't find log file");

    let mut last_log_contents = String::new();

    let mut watcher = Hotwatch::new()?;
    watcher.watch(&latest_log_file, move |event: Event| {
        if let Ok(contents) = fs::read_to_string(&event.paths[0]) {
            if !last_log_contents.is_empty() {
                process_log_diff(&last_log_contents, &contents);
            }
            last_log_contents = contents;
        }

        Flow::Continue
    })?;

    println!("Watching logs file");
    watcher.run();

    Ok(())
}
