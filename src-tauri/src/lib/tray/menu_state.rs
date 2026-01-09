use serde::Serialize;

use crate::state::{is_paused, lock_state, SharedState};

/// State payload sent to the tray menu frontend.
#[derive(Clone, Serialize)]
pub struct TrayMenuState {
    pub enabled: bool,
    pub paused: bool,
}

pub fn menu_state_from(state: &SharedState) -> TrayMenuState {
    let guard = lock_state(state);
    TrayMenuState {
        enabled: guard.config.enabled,
        paused: is_paused(&guard),
    }
}
