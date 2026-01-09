mod commands;
#[cfg(any(target_os = "windows", test))]
mod sanitize;
mod state;
mod toast;
mod tray;
#[cfg(target_os = "windows")]
mod win;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle();
            let config = state::load_config(app_handle);
            let shared_state = state::new_shared_state(config);
            app.manage(shared_state.clone());

            let _ = toast::ensure_toast_window(app_handle);
            tray::init_tray(app_handle, shared_state.clone())?;

            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.hide();
            }

            register_shortcut(app_handle, shared_state.clone())?;

            #[cfg(target_os = "windows")]
            win::clipboard_listener::start_listener(app_handle.clone(), shared_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_ui_state,
            commands::update_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn register_shortcut(app: &AppHandle, state: state::SharedState) -> tauri::Result<()> {
    let state = state.clone();
    app.global_shortcut()
        .on_shortcut("Ctrl+Alt+Shift+V", move |app, _shortcut, _event| {
            tray::set_enabled_from_shortcut(app, &state);
        })
        .map_err(|err| tauri::Error::PluginInitialization("global-shortcut".into(), err.to_string()))?;
    Ok(())
}
