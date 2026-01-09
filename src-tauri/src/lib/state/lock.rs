use std::sync::MutexGuard;

use super::{AppState, SharedState};

pub fn lock_state(shared: &SharedState) -> MutexGuard<'_, AppState> {
    match shared.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
