use crate::MOVABLE;
use hotkey;
use std::thread;

pub fn start_hotkey_thread() -> thread::JoinHandle<()> {
    thread::spawn(|| {
        let mut hk = hotkey::Listener::new();
        hk.register_hotkey(
            hotkey::modifiers::CONTROL | hotkey::modifiers::SHIFT,
            36 as u32,
            || {
                let mut movable = MOVABLE.lock().unwrap();
                *movable = !*movable;
            },
        )
        .unwrap();

        hk.listen();
    })
}
