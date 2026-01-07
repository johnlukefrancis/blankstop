use tauri::{AppHandle, State};

use crate::state::{ui_state, Config, SharedState, UiState};
use crate::tray::apply_config;

#[tauri::command]
pub fn get_ui_state(state: State<SharedState>) -> UiState {
    let guard = state.lock().expect("state mutex poisoned");
    ui_state(&guard)
}

#[tauri::command]
pub fn update_config(
    app: AppHandle,
    state: State<SharedState>,
    config: Config,
) -> Result<UiState, String> {
    apply_config(&app, state.inner(), config)?;
    let guard = state.lock().expect("state mutex poisoned");
    Ok(ui_state(&guard))
}
