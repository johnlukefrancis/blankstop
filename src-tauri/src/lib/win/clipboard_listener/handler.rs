use std::time::{Duration, Instant};

use tauri::AppHandle;
use windows::Win32::Foundation::HWND;

use super::clipboard::{read_clipboard_text, write_clipboard_text};
use crate::sanitize::sanitize_text;
use crate::state::{
    emit_ui_state, hash_text, is_paused, lock_state, now_ms, push_log, set_debug_capture, LogEntry,
    SharedState,
};
use crate::toast::show_toast;
use crate::win::process::{clipboard_owner_exe_name_strict, foreground_exe_name};

const SELF_WRITE_WINDOW_MS: u64 = 700;
const WRITE_FAIL_SUMMARY: &str = "clipboard write failed (locked)";

pub fn handle_clipboard_update(app: &AppHandle, state: &SharedState, hwnd: HWND) {
    let now = Instant::now();
    {
        let guard = lock_state(state);
        if !guard.config.enabled {
            return;
        }
        if is_paused(&guard) {
            return;
        }
        if guard
            .self_write_until
            .map(|until| until > now)
            .unwrap_or(false)
        {
            return;
        }
    }

    let Some(text) = read_clipboard_text(hwnd) else {
        return;
    };

    let text_hash = hash_text(&text);
    {
        let guard = lock_state(state);
        if guard.last_written_hash == Some(text_hash) {
            return;
        }
    }

    let owner_exe = clipboard_owner_exe_name_strict();
    let source_exe = owner_exe.clone().or_else(foreground_exe_name);
    let config = {
        let guard = lock_state(state);
        guard.config.clone()
    };

    if config.only_allowlisted {
        let Some(ref exe) = owner_exe else {
            return;
        };
        if !config.allowlist.contains(exe) {
            return;
        }
    }

    let result = sanitize_text(&text);
    let toast_message = result.summary.toast_message();
    let toast_with_source = format_sanitize_toast(&toast_message, source_exe.as_deref());
    {
        let mut guard = lock_state(state);
        set_debug_capture(&mut guard, &text, &toast_message);
    }
    let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");
    if result.output == normalized_input {
        return;
    }

    let output_for_clipboard = result.output.replace("\n", "\r\n");
    {
        let mut guard = lock_state(state);
        guard.self_write_until = Some(Instant::now() + Duration::from_millis(SELF_WRITE_WINDOW_MS));
    }
    if write_clipboard_text(hwnd, &output_for_clipboard).is_err() {
        let mut guard = lock_state(state);
        apply_write_result(
            &mut guard,
            false,
            0,
            source_exe.clone(),
            WRITE_FAIL_SUMMARY.to_string(),
        );
        emit_ui_state(app, state);
        return;
    }

    {
        let mut guard = lock_state(state);
        apply_write_result(
            &mut guard,
            true,
            hash_text(&output_for_clipboard),
            source_exe.clone(),
            toast_message.clone(),
        );
    }

    emit_ui_state(app, state);
    if config.toast_enabled {
        if let Err(err) = show_toast(app, toast_with_source) {
            let mut guard = lock_state(state);
            push_log(
                &mut guard,
                LogEntry {
                    timestamp_ms: now_ms(),
                    source_exe: source_exe.clone(),
                    summary: format!("Toast failed: {err}"),
                },
            );
        }
    }
}

pub fn sanitize_clipboard_now(app: &AppHandle, state: &SharedState) -> bool {
    let (config, paused) = {
        let guard = lock_state(state);
        (guard.config.clone(), is_paused(&guard))
    };
    if paused {
        return false;
    }
    let owner_exe = clipboard_owner_exe_name_strict();
    let source_exe = owner_exe.clone().or_else(foreground_exe_name);
    if config.only_allowlisted {
        let Some(ref exe) = owner_exe else {
            return false;
        };
        if !config.allowlist.contains(exe) {
            return false;
        }
    }

    let hwnd = HWND(std::ptr::null_mut());
    let Some(text) = read_clipboard_text(hwnd) else {
        return false;
    };

    let result = sanitize_text(&text);
    let toast_message = result.summary.toast_message();
    {
        let mut guard = lock_state(state);
        set_debug_capture(&mut guard, &text, &toast_message);
    }

    let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");
    if result.output == normalized_input {
        return false;
    }

    let output_for_clipboard = result.output.replace("\n", "\r\n");
    {
        let mut guard = lock_state(state);
        guard.self_write_until = Some(Instant::now() + Duration::from_millis(SELF_WRITE_WINDOW_MS));
    }
    if write_clipboard_text(hwnd, &output_for_clipboard).is_err() {
        let mut guard = lock_state(state);
        apply_write_result(
            &mut guard,
            false,
            0,
            source_exe.clone(),
            WRITE_FAIL_SUMMARY.to_string(),
        );
        emit_ui_state(app, state);
        return false;
    }

    {
        let mut guard = lock_state(state);
        apply_write_result(
            &mut guard,
            true,
            hash_text(&output_for_clipboard),
            source_exe.clone(),
            toast_message,
        );
    }

    emit_ui_state(app, state);
    true
}

fn apply_write_result(
    state: &mut crate::state::AppState,
    write_ok: bool,
    written_hash: u64,
    source_exe: Option<String>,
    summary: String,
) {
    let log_source = source_exe.clone();
    if write_ok {
        state.last_written_hash = Some(written_hash);
        state.last_source_exe = source_exe;
    } else {
        state.self_write_until = None;
    }
    push_log(
        state,
        LogEntry {
            timestamp_ms: now_ms(),
            source_exe: log_source,
            summary,
        },
    );
}

fn format_sanitize_toast(_message: &str, source_exe: Option<&str>) -> String {
    match source_exe {
        Some(exe) => format!("✓ Sanitized from {}", exe),
        None => "✓ Clipboard sanitized".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, Config};

    fn base_state() -> AppState {
        AppState {
            config: Config::default(),
            paused_until: None,
            last_written_hash: None,
            self_write_until: Some(Instant::now()),
            last_source_exe: None,
            log: Vec::new(),
            debug_last_clipboard: None,
            debug_last_summary: None,
        }
    }

    #[test]
    fn apply_write_result_clears_guard_on_failure() {
        let mut state = base_state();
        apply_write_result(
            &mut state,
            false,
            123,
            Some("wt.exe".to_string()),
            WRITE_FAIL_SUMMARY.to_string(),
        );
        assert!(state.last_written_hash.is_none());
        assert!(state.self_write_until.is_none());
        assert_eq!(state.log.len(), 1);
        assert_eq!(state.log[0].summary, WRITE_FAIL_SUMMARY);
    }

    #[test]
    fn apply_write_result_sets_hash_on_success() {
        let mut state = base_state();
        apply_write_result(
            &mut state,
            true,
            456,
            Some("code.exe".to_string()),
            "ok".to_string(),
        );
        assert_eq!(state.last_written_hash, Some(456));
        assert!(state.self_write_until.is_some());
        assert_eq!(state.log.len(), 1);
        assert_eq!(state.log[0].summary, "ok");
    }
}
