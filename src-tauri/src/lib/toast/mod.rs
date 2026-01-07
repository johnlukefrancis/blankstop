use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl};

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
    .resizable(false)
    .transparent(true)
    .visible(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .inner_size(360.0, 92.0)
    .build()?;

    position_toast(&window)?;
    Ok(())
}

pub fn show_toast(app: &AppHandle, message: impl Into<String>) {
    let Some(window) = app.get_webview_window("toast") else {
        return;
    };
    let message = message.into();
    let _ = position_toast(&window);
    let _ = window.emit("toast-message", message.clone());
    let _ = window.show();
    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        tauri::async_runtime::sleep(Duration::from_millis(1400)).await;
        let _ = window_clone.hide();
    });
}

fn position_toast(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let size = PhysicalSize::new(360, 92);
    if let Some(monitor) = window.current_monitor()? {
        let monitor_pos = monitor.position();
        let monitor_size = monitor.size();
        let margin = 18i32;
        let x = monitor_pos.x + monitor_size.width as i32 - size.width as i32 - margin;
        let y = monitor_pos.y + monitor_size.height as i32 - size.height as i32 - margin;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}
