#![cfg(target_os = "windows")]

use std::ffi::c_void;
use std::mem::size_of;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use windows::core::w;
use windows::Win32::Foundation::{
    GetLastError, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard,
    RemoveClipboardFormatListener, SetClipboardData, CF_UNICODETEXT,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, GWLP_USERDATA, MSG,
    WM_CLIPBOARDUPDATE, WM_DESTROY, WM_NCCREATE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

use crate::sanitize::sanitize_text;
use crate::state::{
    emit_ui_state, hash_text, now_ms, push_log, is_paused, LogEntry, SharedState,
};
use crate::toast::show_toast;
use crate::win::process::clipboard_owner_exe_name;

const MAX_ATTEMPTS: u32 = 10;
const BACKOFF_MS: u32 = 15;
const SELF_WRITE_WINDOW_MS: u64 = 700;

pub fn start_listener(app: AppHandle, state: SharedState) {
    std::thread::spawn(move || {
        let class_name = w!("BlankstopClipboardListener");
        let hinstance = unsafe { windows::Win32::Foundation::GetModuleHandleW(None) };

        let wnd_class = WNDCLASSW {
            hInstance: hinstance,
            lpszClassName: class_name,
            lpfnWndProc: Some(window_proc),
            ..Default::default()
        };

        unsafe {
            RegisterClassW(&wnd_class);
        }

        let ctx = Box::new(ListenerContext { app, state });
        let ctx_ptr = Box::into_raw(ctx) as *mut c_void;

        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                class_name,
                w!(""),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                None,
                hinstance,
                ctx_ptr,
            )
        };

        if hwnd.0 == 0 {
            let _ = unsafe { GetLastError() };
            unsafe { Box::from_raw(ctx_ptr as *mut ListenerContext) };
            return;
        }

        unsafe {
            let _ = AddClipboardFormatListener(hwnd);
        }

        let mut message = MSG::default();
        unsafe {
            while GetMessageW(&mut message, HWND(0), 0, 0).as_bool() {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    });
}

struct ListenerContext {
    app: AppHandle,
    state: SharedState,
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
            let ctx_ptr = create_struct.lpCreateParams as *mut ListenerContext;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ctx_ptr as isize);
            LRESULT(1)
        }
        WM_CLIPBOARDUPDATE => {
            let ctx = get_context(hwnd);
            if let Some(ctx) = ctx {
                handle_clipboard_update(&ctx.app, &ctx.state, hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = RemoveClipboardFormatListener(hwnd);
            let ctx = get_context(hwnd);
            if let Some(ctx) = ctx {
                let _ = Box::from_raw(ctx as *const ListenerContext as *mut ListenerContext);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn get_context(hwnd: HWND) -> Option<&'static ListenerContext> {
    let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if ptr == 0 {
        None
    } else {
        Some(&*(ptr as *const ListenerContext))
    }
}

fn handle_clipboard_update(app: &AppHandle, state: &SharedState, hwnd: HWND) {
    let now = Instant::now();
    {
        let guard = state.lock().expect("state mutex poisoned");
        if !guard.config.enabled {
            return;
        }
        if is_paused(&guard) {
            return;
        }
        if guard
            .self_write_until
            .map(|until| until > now)
            .unwrap_or(false)
        {
            return;
        }
    }

    let Some(text) = read_clipboard_text(hwnd) else {
        return;
    };

    let text_hash = hash_text(&text);
    {
        let guard = state.lock().expect("state mutex poisoned");
        if guard.last_written_hash == Some(text_hash) {
            return;
        }
    }

    let source_exe = clipboard_owner_exe_name();
    let config = {
        let guard = state.lock().expect("state mutex poisoned");
        guard.config.clone()
    };

    if config.only_allowlisted {
        let Some(ref exe) = source_exe else {
            return;
        };
        if !config.allowlist.contains(exe) {
            return;
        }
    }

    let result = sanitize_text(&text);
    let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");
    if result.output == normalized_input {
        return;
    }

    let output_for_clipboard = result.output.replace("\n", "\r\n");
    if write_clipboard_text(hwnd, &output_for_clipboard).is_err() {
        return;
    }

    let toast_message = result.summary.toast_message();
    {
        let mut guard = state.lock().expect("state mutex poisoned");
        guard.last_written_hash = Some(hash_text(&output_for_clipboard));
        guard.self_write_until = Some(Instant::now() + Duration::from_millis(SELF_WRITE_WINDOW_MS));
        guard.last_source_exe = source_exe.clone();
        push_log(
            &mut guard,
            LogEntry {
                timestamp_ms: now_ms(),
                source_exe,
                summary: toast_message.clone(),
            },
        );
    }

    emit_ui_state(app, state);
    if config.toast_enabled {
        show_toast(app, toast_message);
    }
}

fn read_clipboard_text(hwnd: HWND) -> Option<String> {
    for attempt in 0..MAX_ATTEMPTS {
        if !unsafe { OpenClipboard(hwnd).as_bool() } {
            unsafe { Sleep(BACKOFF_MS * (attempt + 1)) };
            continue;
        }
        let handle = unsafe { GetClipboardData(CF_UNICODETEXT) };
        if handle.0 == 0 {
            unsafe { CloseClipboard() };
            return None;
        }
        let text = unsafe { copy_hglobal_utf16(handle) };
        unsafe { CloseClipboard() };
        return text;
    }
    None
}

fn write_clipboard_text(hwnd: HWND, text: &str) -> Result<(), ()> {
    for attempt in 0..MAX_ATTEMPTS {
        if !unsafe { OpenClipboard(hwnd).as_bool() } {
            unsafe { Sleep(BACKOFF_MS * (attempt + 1)) };
            continue;
        }

        let _ = unsafe { EmptyClipboard() };
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size_bytes = wide.len() * size_of::<u16>();
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, size_bytes) };
        if handle.0 == 0 {
            unsafe { CloseClipboard() };
            return Err(());
        }
        let lock = unsafe { GlobalLock(handle) } as *mut u16;
        if lock.is_null() {
            unsafe {
                let _ = GlobalFree(handle);
                CloseClipboard();
            }
            return Err(());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(wide.as_ptr(), lock, wide.len());
            GlobalUnlock(handle);
        }

        let set_ok = unsafe { SetClipboardData(CF_UNICODETEXT, handle) };
        unsafe { CloseClipboard() };
        if set_ok.0 == 0 {
            unsafe { GlobalFree(handle) };
            return Err(());
        }
        return Ok(());
    }
    Err(())
}

unsafe fn copy_hglobal_utf16(handle: HGLOBAL) -> Option<String> {
    let ptr = GlobalLock(handle) as *const u16;
    if ptr.is_null() {
        return None;
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    let text = String::from_utf16_lossy(slice);
    let _ = GlobalUnlock(handle);
    Some(text)
}

const HWND_MESSAGE: HWND = HWND(-3);
