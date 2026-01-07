use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::Config;

pub fn config_path(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&base);
    base.join("config.json")
}

pub fn load_config(app: &AppHandle) -> Config {
    let path = config_path(app);
    let data = fs::read_to_string(path).ok();
    let mut config = data
        .and_then(|raw| serde_json::from_str::<Config>(&raw).ok())
        .unwrap_or_default();
    config.normalize();
    config
}

pub fn save_config(app: &AppHandle, config: &Config) -> Result<(), String> {
    let path = config_path(app);
    let raw = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    fs::write(path, raw).map_err(|err| err.to_string())
}
