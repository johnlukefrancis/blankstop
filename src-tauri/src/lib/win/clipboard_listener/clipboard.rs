use std::mem::size_of;

use windows::Win32::Foundation::{GlobalFree, HGLOBAL, HANDLE, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::System::Threading::Sleep;

const MAX_ATTEMPTS: u32 = 10;
const BACKOFF_MS: u32 = 15;

pub fn read_clipboard_text(hwnd: HWND) -> Option<String> {
    for attempt in 0..MAX_ATTEMPTS {
        if unsafe { OpenClipboard(hwnd).is_err() } {
            unsafe { Sleep(BACKOFF_MS * (attempt + 1)) };
            continue;
        }
        let handle = unsafe { GetClipboardData(CF_UNICODETEXT.0 as u32) }.ok()?;
        if handle.0.is_null() {
            let _ = unsafe { CloseClipboard() };
            return None;
        }
        let text = unsafe { copy_hglobal_utf16(HGLOBAL(handle.0)) };
        let _ = unsafe { CloseClipboard() };
        return text;
    }
    None
}

pub fn write_clipboard_text(hwnd: HWND, text: &str) -> Result<(), ()> {
    for attempt in 0..MAX_ATTEMPTS {
        if unsafe { OpenClipboard(hwnd).is_err() } {
            unsafe { Sleep(BACKOFF_MS * (attempt + 1)) };
            continue;
        }

        let _ = unsafe { EmptyClipboard() };
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size_bytes = wide.len() * size_of::<u16>();
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, size_bytes) }.ok();
        let handle = match handle {
            Some(handle) if !handle.0.is_null() => handle,
            _ => {
                let _ = unsafe { CloseClipboard() };
                return Err(());
            }
        };
        let lock = unsafe { GlobalLock(handle) } as *mut u16;
        if lock.is_null() {
            unsafe {
                let _ = GlobalFree(handle);
                let _ = CloseClipboard();
            }
            return Err(());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(wide.as_ptr(), lock, wide.len());
            let _ = GlobalUnlock(handle);
        }

        let set_ok = unsafe { SetClipboardData(CF_UNICODETEXT.0 as u32, HANDLE(handle.0)) };
        let _ = unsafe { CloseClipboard() };
        if set_ok.is_err() {
            unsafe { let _ = GlobalFree(handle); };
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
