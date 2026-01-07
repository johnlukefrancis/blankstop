#![cfg(target_os = "windows")]

use std::path::Path;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::DataExchange::GetClipboardOwner;
use windows::Win32::System::Threading::{
    GetWindowThreadProcessId, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

pub fn clipboard_owner_exe_name() -> Option<String> {
    unsafe {
        let mut hwnd = GetClipboardOwner().ok()?;
        if hwnd.0.is_null() {
            hwnd = GetForegroundWindow();
        }
        if hwnd.0.is_null() {
            return None;
        }
        exe_name_from_hwnd(hwnd)
    }
}

fn exe_name_from_hwnd(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        if handle.is_invalid() {
            return None;
        }
        let mut buffer = vec![0u16; 260];
        let mut len = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, PWSTR(buffer.as_mut_ptr()), &mut len);
        let _ = CloseHandle(handle);
        if ok.is_err() || len == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buffer[..len as usize]);
        let file = Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase());
        file
    }
}
