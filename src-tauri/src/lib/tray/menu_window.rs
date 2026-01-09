use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl};

use super::menu_state::menu_state_from;
use crate::state::SharedState;

const MENU_WIDTH: u32 = 200;
const MENU_HEIGHT: u32 = 220;
const MENU_MARGIN: i32 = 8;

pub fn ensure_menu_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("tray-menu").is_some() {
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(app, "tray-menu", WebviewUrl::App("index.html".into()))
        .initialization_script("window.__blankstopTrayMenu = true;")
        .decorations(false)
        .shadow(true)
        .resizable(false)
        .focusable(true)
        .focused(false)
        .transparent(true)
        .visible(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .inner_size(MENU_WIDTH as f64, MENU_HEIGHT as f64)
        .build()?;
    Ok(())
}

pub fn show_menu(app: &AppHandle, state: &SharedState, click_pos: PhysicalPosition<f64>) {
    let Some(window) = app.get_webview_window("tray-menu") else {
        return;
    };
    let _ = position_menu(&window, click_pos);
    let menu_state = menu_state_from(state);
    let _ = window.emit("tray-menu-state", menu_state);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn hide_menu(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("tray-menu") {
        let _ = window.hide();
    }
}

fn position_menu(window: &tauri::WebviewWindow, click_pos: PhysicalPosition<f64>) -> tauri::Result<()> {
    let size = PhysicalSize::new(MENU_WIDTH as i32, MENU_HEIGHT as i32);
    if let Some(monitor) = window.current_monitor()? {
        let work_area = monitor.work_area();
        let work_right = work_area.position.x + work_area.size.width as i32;
        let work_bottom = work_area.position.y + work_area.size.height as i32;

        // Position above the click point, centered horizontally
        let mut x = click_pos.x as i32 - size.width / 2;
        let mut y = click_pos.y as i32 - size.height - MENU_MARGIN;

        // Clamp to work area bounds
        x = x.max(work_area.position.x).min(work_right - size.width);
        y = y.max(work_area.position.y).min(work_bottom - size.height);

        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}
