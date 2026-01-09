use tauri::AppHandle;

use crate::state::SharedState;

mod clipboard;
mod handler;
mod window;
mod worker;

pub fn start_listener(app: AppHandle, state: SharedState) {
    window::start_listener(app, state);
}

pub fn sanitize_clipboard_now(app: &AppHandle, state: &SharedState) -> bool {
    handler::sanitize_clipboard_now(app, state)
}
