use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use super::Config;

pub fn config_file_path(app: &AppHandle) -> Option<PathBuf> {
    let base = app.path().app_config_dir().ok()?;
    Some(base.join("config.json"))
}

pub fn load_config(app: &AppHandle) -> Result<Config, String> {
    let Some(path) = config_file_path(app) else {
        let mut config = Config::default();
        config.normalize();
        return Ok(config);
    };
    let data = fs::read(&path).ok();
    let mut config = match data {
        Some(raw) => match serde_json::from_slice::<Config>(&raw) {
            Ok(parsed) => parsed,
            Err(_) => {
                backup_corrupt_config(&path);
                Config::default()
            }
        },
        None => Config::default(),
    };
    config.normalize();
    Ok(config)
}

pub fn save_config(app: &AppHandle, config: &Config) -> Result<(), String> {
    let Some(path) = config_file_path(app) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let raw = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    fs::write(path, raw).map_err(|err| err.to_string())
}

fn backup_corrupt_config(path: &Path) {
    let timestamp = now_ms();
    let filename = format!("config.json.bad-{timestamp}");
    let backup_path = path.with_file_name(filename);
    if fs::rename(path, &backup_path).is_ok() {
        return;
    }
    if fs::copy(path, &backup_path).is_ok() {
        let _ = fs::remove_file(path);
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}
