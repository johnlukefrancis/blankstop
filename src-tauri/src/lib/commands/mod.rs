use tauri::{AppHandle, State};

use crate::state::{lock_state, ui_state, Config, SharedState, UiState};
use crate::tray::{apply_config, handle_menu_action, menu_state, menu_window};

#[tauri::command]
pub fn get_ui_state(state: State<SharedState>) -> UiState {
    let guard = lock_state(state.inner());
    ui_state(&guard)
}

#[tauri::command]
pub fn update_config(
    app: AppHandle,
    state: State<SharedState>,
    config: Config,
) -> Result<UiState, String> {
    apply_config(&app, state.inner(), config);
    let guard = lock_state(state.inner());
    Ok(ui_state(&guard))
}

#[tauri::command]
pub fn tray_menu_action(app: AppHandle, state: State<SharedState>, action: String) {
    handle_menu_action(&app, state.inner(), &action);
}

#[tauri::command]
pub fn hide_tray_menu(app: AppHandle) {
    menu_window::hide_menu(&app);
}

#[tauri::command]
pub fn get_tray_menu_state(state: State<SharedState>) -> menu_state::TrayMenuState {
    menu_state::menu_state_from(state.inner())
}
