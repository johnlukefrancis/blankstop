use std::time::{Duration, Instant};

use tauri::AppHandle;
use windows::Win32::Foundation::HWND;

use super::clipboard::{read_clipboard_text, write_clipboard_text};
use crate::sanitize::sanitize_text;
use crate::state::{
    emit_ui_state, hash_text, is_paused, now_ms, push_log, set_debug_capture, LogEntry,
    SharedState,
};
use crate::toast::show_toast;
use crate::win::process::{clipboard_owner_exe_name_strict, foreground_exe_name};

const SELF_WRITE_WINDOW_MS: u64 = 700;

pub fn handle_clipboard_update(app: &AppHandle, state: &SharedState, hwnd: HWND) {
    let now = Instant::now();
    {
        let guard = state.lock().expect("state mutex poisoned");
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
        let guard = state.lock().expect("state mutex poisoned");
        if guard.last_written_hash == Some(text_hash) {
            return;
        }
    }

    let owner_exe = clipboard_owner_exe_name_strict();
    let source_exe = owner_exe.clone().or_else(foreground_exe_name);
    let config = {
        let guard = state.lock().expect("state mutex poisoned");
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
        let mut guard = state.lock().expect("state mutex poisoned");
        set_debug_capture(&mut guard, &text, &toast_message);
    }
    let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");
    if result.output == normalized_input {
        return;
    }

    let output_for_clipboard = result.output.replace("\n", "\r\n");
    if write_clipboard_text(hwnd, &output_for_clipboard).is_err() {
        return;
    }

    {
        let mut guard = state.lock().expect("state mutex poisoned");
        guard.last_written_hash = Some(hash_text(&output_for_clipboard));
        guard.self_write_until = Some(Instant::now() + Duration::from_millis(SELF_WRITE_WINDOW_MS));
        guard.last_source_exe = source_exe.clone();
        push_log(
            &mut guard,
            LogEntry {
                timestamp_ms: now_ms(),
                source_exe,
                summary: toast_message.clone(),
            },
        );
    }

    emit_ui_state(app, state);
    if config.toast_enabled {
        show_toast(app, toast_with_source);
    }
}

pub fn sanitize_clipboard_now(app: &AppHandle, state: &SharedState) -> bool {
    let (config, paused) = {
        let guard = state.lock().expect("state mutex poisoned");
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
        let mut guard = state.lock().expect("state mutex poisoned");
        set_debug_capture(&mut guard, &text, &toast_message);
    }

    let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");
    if result.output == normalized_input {
        return false;
    }

    let output_for_clipboard = result.output.replace("\n", "\r\n");
    if write_clipboard_text(hwnd, &output_for_clipboard).is_err() {
        return false;
    }

    {
        let mut guard = state.lock().expect("state mutex poisoned");
        guard.last_written_hash = Some(hash_text(&output_for_clipboard));
        guard.self_write_until = Some(Instant::now() + Duration::from_millis(SELF_WRITE_WINDOW_MS));
        guard.last_source_exe = source_exe.clone();
        push_log(
            &mut guard,
            LogEntry {
                timestamp_ms: now_ms(),
                source_exe,
                summary: toast_message,
            },
        );
    }

    emit_ui_state(app, state);
    true
}

fn format_sanitize_toast(message: &str, source_exe: Option<&str>) -> String {
    match source_exe {
        Some(exe) => format!("{} ({})", message, exe),
        None => message.to_string(),
    }
}
