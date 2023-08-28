use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::file_watcher::start_file_watcher_thread;
use crate::hotkeys::start_hotkey_thread;
use crate::ui::Overlay;

mod file_watcher;
mod files;
mod hotkeys;
mod ui;

pub struct Player {
    name: String,
}

lazy_static! {
    pub static ref NEARBY_PLAYERS: Mutex<HashMap<String, Player>> = Mutex::new(HashMap::new());
    pub static ref MOVABLE: Mutex<bool> = Mutex::new(true);
}

fn main() {
    // Spawn the file watcher thread
    println!("Starting file watcher");
    let file_watcher = start_file_watcher_thread();

    // Spawn the hotkey thread
    println!("Starting hotkeys");
    let hotkey_thread = start_hotkey_thread();

    // Run the UI on the main thread
    println!("Starting overlay");
    egui_overlay::start(Overlay {});

    file_watcher.join().expect("File watcher thread panicked");
    hotkey_thread.join().expect("Hotkey thread panicked");
}
