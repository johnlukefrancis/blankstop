mod config;
#[cfg(target_os = "windows")]
mod hash;
mod io;
mod lock;
mod models;
mod runtime;
#[cfg(target_os = "windows")]
mod time;

pub use config::Config;
pub use io::{load_config, save_config};
pub use lock::lock_state;
pub use models::{AppState, LogEntry, UiState};
pub use runtime::{emit_ui_state, is_paused, new_shared_state, ui_state, SharedState};

#[cfg(target_os = "windows")]
pub use hash::hash_text;
#[cfg(target_os = "windows")]
pub use runtime::push_log;
#[cfg(target_os = "windows")]
pub use runtime::set_debug_capture;
#[cfg(target_os = "windows")]
pub use time::now_ms;
