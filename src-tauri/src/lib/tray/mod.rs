use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, Wry};

use crate::state::{emit_ui_state, is_paused, save_config, Config, SharedState};
use crate::toast::show_toast;
use crate::win::clipboard_listener::sanitize_clipboard_now;

pub fn init_tray(app: &AppHandle, state: SharedState) -> tauri::Result<()> {
    let menu = build_menu(app, &state)?;
    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().unwrap())
        .tooltip("Blankstop")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "enabled" => {
                    toggle_enabled(app, &state, true);
                }
                "pause_5" => {
                    set_paused(&state, Some(Duration::from_secs(300)));
                    sync_menu(app, &state);
                    emit_ui_state(app, &state);
                }
                "resume" => {
                    set_paused(&state, None);
                    sync_menu(app, &state);
                    emit_ui_state(app, &state);
                }
                "settings" => {
                    open_settings_window(app, &state);
                }
                "sanitize_now" => {
                    let changed = sanitize_clipboard_now(app, &state);
                    let message = if changed {
                        "Sanitize now: changed"
                    } else {
                        "Sanitize now: no changes"
                    };
                    show_toast(app, message);
                }
                "test_toast" => {
                    show_toast(app, "Blankstop test toast");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

pub fn sync_menu(app: &AppHandle, state: &SharedState) {
    let Ok(menu) = build_menu(app, state) else {
        return;
    };
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_menu(Some(menu));
    }
}

fn build_menu(app: &AppHandle, state: &SharedState) -> tauri::Result<Menu<Wry>> {
    let (config, paused) = {
        let guard = state.lock().expect("state mutex poisoned");
        (guard.config.clone(), is_paused(&guard))
    };
    let enabled_item = CheckMenuItem::with_id(
        app,
        "enabled",
        "Enabled",
        true,
        config.enabled,
        None::<&str>,
    )?;
    let pause_item = MenuItem::with_id(app, "pause_5", "Pause 5 min", true, None::<&str>)?;
    let resume_item = MenuItem::with_id(app, "resume", "Resume now", paused, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let sanitize_now_item =
        MenuItem::with_id(app, "sanitize_now", "Sanitize clipboard now", true, None::<&str>)?;
    let test_toast_item = MenuItem::with_id(app, "test_toast", "Test Toast", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &enabled_item,
            &pause_item,
            &resume_item,
            &settings_item,
            &sanitize_now_item,
            &test_toast_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )
}

fn toggle_enabled(app: &AppHandle, state: &SharedState, show_toast_notification: bool) {
    let (config, source_exe) = {
        let mut guard = state.lock().expect("state mutex poisoned");
        guard.config.enabled = !guard.config.enabled;
        (guard.config.clone(), guard.last_source_exe.clone())
    };
    if save_config(app, &config).is_ok() {
        sync_menu(app, state);
        emit_ui_state(app, state);
        if show_toast_notification && config.status_toast_enabled {
            let message = format_enabled_toast(config.enabled, source_exe.as_deref());
            show_toast(app, message);
        }
    }
}

fn set_paused(state: &SharedState, duration: Option<Duration>) {
    let mut guard = state.lock().expect("state mutex poisoned");
    guard.paused_until = duration.map(|d| Instant::now() + d);
}

pub fn set_enabled_from_shortcut(app: &AppHandle, state: &SharedState) {
    toggle_enabled(app, state, true);
}

pub fn apply_config(app: &AppHandle, state: &SharedState, mut config: Config) -> Result<(), String> {
    config.normalize();
    {
        let mut guard = state.lock().expect("state mutex poisoned");
        guard.config = config.clone();
    }
    save_config(app, &config)?;
    sync_menu(app, state);
    emit_ui_state(app, state);
    Ok(())
}

fn open_settings_window(app: &AppHandle, state: &SharedState) {
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

fn format_enabled_toast(enabled: bool, source_exe: Option<&str>) -> String {
    let base = if enabled {
        "Blankstop enabled"
    } else {
        "Blankstop disabled"
    };
    match source_exe {
        Some(exe) => format!("{} ({})", base, exe),
        None => base.to_string(),
    }
}
