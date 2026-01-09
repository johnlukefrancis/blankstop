use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl};

static TOAST_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn ensure_toast_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("toast").is_some() {
        return Ok(());
    }
    let window = tauri::WebviewWindowBuilder::new(
        app,
        "toast",
        WebviewUrl::App("index.html".into()),
    )
    .initialization_script("window.__blankstopToast = true;")
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .focusable(false)
    .focused(false)
    .transparent(true)
    .visible(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .inner_size(320.0, 48.0)
    .build()?;

    let _ = window.set_ignore_cursor_events(true);
    let _ = window.set_focusable(false);
    position_toast(&window)?;
    Ok(())
}

pub fn show_toast(app: &AppHandle, message: impl Into<String>) {
    let Some(window) = app.get_webview_window("toast") else {
        return;
    };
    let message = message.into();
    let generation = TOAST_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = position_toast(&window);
    let _ = window.emit("toast-message", message.clone());
    let _ = window.show();
    let window_clone = window.clone();
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(1400));
        if TOAST_GENERATION.load(Ordering::Relaxed) == generation {
            let _ = window_clone.hide();
        }
    });
}

fn position_toast(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let size = PhysicalSize::new(320, 48);
    if let Some(monitor) = window.current_monitor()? {
        let work_area = monitor.work_area();
        let margin = 18i32;
        let x = work_area.position.x + work_area.size.width as i32 - size.width - margin;
        let y = work_area.position.y + work_area.size.height as i32 - size.height - margin;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}
