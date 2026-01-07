use std::ffi::c_void;

use tauri::AppHandle;
use windows::core::w;
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, GWLP_USERDATA, HWND_MESSAGE,
    MSG, WM_CLIPBOARDUPDATE, WM_DESTROY, WM_NCCREATE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

use super::handler::handle_clipboard_update;
use crate::state::SharedState;

pub fn start_listener(app: AppHandle, state: SharedState) {
    std::thread::spawn(move || {
        let class_name = w!("BlankstopClipboardListener");
        let hinstance = unsafe { GetModuleHandleW(None) }.ok();

        let Some(hinstance) = hinstance else {
            return;
        };

        let wnd_class = WNDCLASSW {
            hInstance: hinstance.into(),
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
                Some(ctx_ptr as *const c_void),
            )
        }
        .ok();

        let Some(hwnd) = hwnd else {
            let _ = unsafe { GetLastError() };
            unsafe { drop(Box::from_raw(ctx_ptr as *mut ListenerContext)) };
            return;
        };

        let _ = unsafe { AddClipboardFormatListener(hwnd) };

        let mut message = MSG::default();
        unsafe {
            while GetMessageW(&mut message, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&message);
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
