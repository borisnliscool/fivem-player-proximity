use std::{env, fs, path::PathBuf};

pub fn get_logs_folder() -> Result<String, env::VarError> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let subfolder_path = PathBuf::from(local_app_data).join("FiveM/FiveM.app/logs");
    Ok(subfolder_path.to_string_lossy().into_owned())
}

pub fn get_latest_log_path(logs_folder: &str) -> Option<PathBuf> {
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
