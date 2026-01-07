use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Emitter};

use super::{AppState, Config, LogEntry, UiState};

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
    let _ = app.emit("ui-state", payload);
}
