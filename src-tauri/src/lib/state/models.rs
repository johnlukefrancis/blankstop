use serde::Serialize;
use std::time::Instant;

use super::Config;

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
