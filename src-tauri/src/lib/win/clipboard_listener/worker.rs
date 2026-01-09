use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};

use tauri::AppHandle;
use windows::Win32::Foundation::HWND;

use super::handler;
use crate::state::{lock_state, now_ms, push_log, LogEntry, SharedState};

pub fn start_worker(app: AppHandle, state: SharedState) -> SyncSender<isize> {
    let (tx, rx) = sync_channel(1);
    std::thread::spawn(move || worker_loop(app, state, rx));
    tx
}

fn worker_loop(app: AppHandle, state: SharedState, rx: Receiver<isize>) {
    while let Ok(mut hwnd_raw) = rx.recv() {
        loop {
            match rx.try_recv() {
                Ok(next) => hwnd_raw = next,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        let hwnd = HWND(hwnd_raw as *mut _);
        let result = catch_unwind(AssertUnwindSafe(|| {
            handler::handle_clipboard_update(&app, &state, hwnd);
        }));
        if result.is_err() {
            let mut guard = lock_state(&state);
            push_log(
                &mut guard,
                LogEntry {
                    timestamp_ms: now_ms(),
                    source_exe: None,
                    summary: "Clipboard update handler panicked; recovered".to_string(),
                },
            );
        }
    }
}
