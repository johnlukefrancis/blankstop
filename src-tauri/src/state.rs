use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub toast_enabled: bool,
    pub only_allowlisted: bool,
    pub allowlist: Vec<String>,
    pub run_on_startup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            toast_enabled: true,
            only_allowlisted: true,
            allowlist: vec![
                "windowsterminal.exe".to_string(),
                "wt.exe".to_string(),
                "code.exe".to_string(),
                "conhost.exe".to_string(),
                "pwsh.exe".to_string(),
                "powershell.exe".to_string(),
                "cmd.exe".to_string(),
                "bash.exe".to_string(),
                "wsl.exe".to_string(),
            ],
            run_on_startup: false,
        }
    }
}

impl Config {
    pub fn normalize(&mut self) {
        self.allowlist = normalize_allowlist(&self.allowlist);
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub source_exe: Option<String>,
    pub summary: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct UiState {
    pub config: Config,
    pub last_source_exe: Option<String>,
    pub log: Vec<LogEntry>,
}

pub struct AppState {
    pub config: Config,
    pub paused_until: Option<Instant>,
    pub last_written_hash: Option<u64>,
    pub self_write_until: Option<Instant>,
    pub last_source_exe: Option<String>,
    pub log: Vec<LogEntry>,
}

pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_shared_state(config: Config) -> SharedState {
    Arc::new(Mutex::new(AppState {
        config,
        paused_until: None,
        last_written_hash: None,
        self_write_until: None,
        last_source_exe: None,
        log: Vec::new(),
    }))
}

pub fn is_paused(state: &AppState) -> bool {
    state
        .paused_until
        .map(|until| until > Instant::now())
        .unwrap_or(false)
}

pub fn push_log(state: &mut AppState, entry: LogEntry) {
    state.log.insert(0, entry);
    if state.log.len() > 10 {
        state.log.truncate(10);
    }
}

pub fn ui_state(state: &AppState) -> UiState {
    UiState {
        config: state.config.clone(),
        last_source_exe: state.last_source_exe.clone(),
        log: state.log.clone(),
    }
}

pub fn emit_ui_state(app: &AppHandle, shared: &SharedState) {
    let state = shared.lock().expect("state mutex poisoned");
    let payload = ui_state(&state);
    let _ = app.emit_all("ui-state", payload);
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

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

pub fn normalize_allowlist(list: &[String]) -> Vec<String> {
    let mut set = HashSet::new();
    let mut out = Vec::new();
    for entry in list {
        let trimmed = entry.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if set.insert(trimmed.clone()) {
            out.push(trimmed);
        }
    }
    out
}

pub fn hash_text(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
