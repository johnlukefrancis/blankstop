use tauri::AppHandle;

use crate::state::SharedState;

mod clipboard;
mod handler;
mod window;

pub fn start_listener(app: AppHandle, state: SharedState) {
    window::start_listener(app, state);
}
