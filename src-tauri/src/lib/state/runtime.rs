use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Emitter};

use super::{lock_state, AppState, Config, LogEntry, UiState};

pub type SharedState = Arc<Mutex<AppState>>;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const DEBUG_MAX_CHARS: usize = 4096;

pub fn new_shared_state(config: Config) -> SharedState {
    Arc::new(Mutex::new(AppState {
        config,
        paused_until: None,
        last_written_hash: None,
        self_write_until: None,
        last_source_exe: None,
        log: Vec::new(),
        debug_last_clipboard: None,
        debug_last_summary: None,
    }))
}

pub fn is_paused(state: &AppState) -> bool {
    state
        .paused_until
        .map(|until| until > Instant::now())
        .unwrap_or(false)
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
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
        debug_last_clipboard: state.debug_last_clipboard.clone(),
        debug_last_summary: state.debug_last_summary.clone(),
    }
}

pub fn emit_ui_state(app: &AppHandle, shared: &SharedState) {
    let state = lock_state(shared);
    let payload = ui_state(&state);
    let _ = app.emit("ui-state", payload);
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn set_debug_capture(state: &mut AppState, raw: &str, summary: &str) {
    state.debug_last_clipboard = Some(truncate_debug(raw));
    state.debug_last_summary = Some(truncate_debug(summary));
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn truncate_debug(value: &str) -> String {
    value.chars().take(DEBUG_MAX_CHARS).collect()
}
