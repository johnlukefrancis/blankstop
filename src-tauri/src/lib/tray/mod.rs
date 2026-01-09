pub mod menu_state;
pub mod menu_window;

use std::time::{Duration, Instant};

use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, Wry};

use crate::state::{
    emit_ui_state, lock_state, now_ms, push_log, save_config, Config, LogEntry, SharedState,
};
use crate::toast::show_toast;
#[cfg(target_os = "windows")]
use crate::win::clipboard_listener::sanitize_clipboard_now;

#[cfg(not(target_os = "windows"))]
fn sanitize_clipboard_now(_app: &AppHandle, _state: &SharedState) -> bool {
    false
}

pub fn init_tray(app: &AppHandle, state: SharedState) -> tauri::Result<()> {
    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| tauri::include_image!("icons/128x128.png").clone());
    let state_clone = state.clone();
    TrayIconBuilder::<Wry>::with_id("main")
        .icon(icon)
        .tooltip("Blankstop")
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: tauri::tray::MouseButtonState::Up,
                position,
                ..
            } = event
            {
                menu_window::show_menu(tray.app_handle(), &state_clone, position);
            }
        })
        .build(app)?;
    Ok(())
}

pub fn handle_menu_action(app: &AppHandle, state: &SharedState, action: &str) {
    match action {
        "enabled" => {
            toggle_enabled(app, state, true);
        }
        "pause_5" => {
            set_paused(state, Some(Duration::from_secs(300)));
            emit_ui_state(app, state);
        }
        "settings" => {
            open_settings_window(app, state);
        }
        "sanitize_now" => {
            let changed = sanitize_clipboard_now(app, state);
            let message = if changed {
                "Sanitize now: changed"
            } else {
                "Sanitize now: no changes"
            };
            show_toast(app, message);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
    // Hide menu after action (except settings which opens a window)
    if action != "settings" {
        menu_window::hide_menu(app);
    }
}

fn toggle_enabled(app: &AppHandle, state: &SharedState, show_toast_notification: bool) {
    let (config, source_exe) = {
        let mut guard = lock_state(state);
        guard.config.enabled = !guard.config.enabled;
        (guard.config.clone(), guard.last_source_exe.clone())
    };
    emit_ui_state(app, state);
    if show_toast_notification && config.status_toast_enabled {
        let message = format_enabled_toast(config.enabled, source_exe.as_deref());
        show_toast(app, message);
    }
    if let Err(err) = save_config(app, &config) {
        log_save_error(state, err);
    }
}

fn set_paused(state: &SharedState, duration: Option<Duration>) {
    let mut guard = lock_state(state);
    guard.paused_until = duration.map(|d| Instant::now() + d);
}

pub fn set_enabled_from_shortcut(app: &AppHandle, state: &SharedState) {
    toggle_enabled(app, state, true);
}

pub fn apply_config(app: &AppHandle, state: &SharedState, mut config: Config) {
    config.normalize();
    {
        let mut guard = lock_state(state);
        guard.config = config.clone();
    }
    emit_ui_state(app, state);
    if let Err(err) = save_config(app, &config) {
        log_save_error(state, err);
    }
}

fn open_settings_window(app: &AppHandle, state: &SharedState) {
    // Hide menu first when opening settings
    menu_window::hide_menu(app);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        emit_ui_state(app, state);
        return;
    }
    let window = tauri::WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Blankstop Settings")
        .inner_size(560.0, 560.0)
        .resizable(false)
        .visible(true)
        .build();
    if let Ok(window) = window {
        let _ = window.set_focus();
        emit_ui_state(app, state);
    }
}

fn format_enabled_toast(enabled: bool, _source_exe: Option<&str>) -> String {
    if enabled {
        "✓ Blankstop enabled".to_string()
    } else {
        "✗ Blankstop disabled".to_string()
    }
}

fn log_save_error(state: &SharedState, err: String) {
    let mut guard = lock_state(state);
    push_log(
        &mut guard,
        LogEntry {
            timestamp_ms: now_ms(),
            source_exe: None,
            summary: format!("Config save failed: {err}"),
        },
    );
}
