use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
