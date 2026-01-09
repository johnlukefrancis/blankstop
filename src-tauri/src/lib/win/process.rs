#![cfg(target_os = "windows")]

use windows::core::PWSTR;
use windows::Win32::Foundation::{
    CloseHandle, HANDLE, HWND, ERROR_INSUFFICIENT_BUFFER, ERROR_MORE_DATA,
};
use windows::Win32::System::DataExchange::GetClipboardOwner;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId,
};

pub fn clipboard_owner_exe_name_strict() -> Option<String> {
    unsafe {
        let hwnd = GetClipboardOwner().ok()?;
        if hwnd.0.is_null() {
            return None;
        }
        exe_name_from_hwnd(hwnd)
    }
}

pub fn foreground_exe_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
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
        let path = query_full_process_image_name(handle);
        let _ = CloseHandle(handle);
        let path = path?;
        basename_from_path(&path)
    }
}

fn query_full_process_image_name(handle: HANDLE) -> Option<String> {
    const START_CAPACITY: usize = 260;
    const MAX_CAPACITY: usize = 32 * 1024;
    let mut capacity = START_CAPACITY;
    loop {
        let mut buffer = vec![0u16; capacity];
        let mut len = buffer.len() as u32;
        let result = unsafe {
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(buffer.as_mut_ptr()),
                &mut len,
            )
        };
        if result.is_ok() && len > 0 {
            return Some(String::from_utf16_lossy(&buffer[..len as usize]));
        }
        if capacity >= MAX_CAPACITY {
            return None;
        }
        if let Err(err) = result {
            let code = err.code();
            if code != ERROR_INSUFFICIENT_BUFFER.into()
                && code != ERROR_MORE_DATA.into()
            {
                return None;
            }
        } else {
            return None;
        }
        capacity = (capacity * 2).min(MAX_CAPACITY);
    }
}

fn basename_from_path(path: &str) -> Option<String> {
    let name = path
        .rsplit(|c| c == '\\' || c == '/')
        .find(|segment| !segment.is_empty())?;
    Some(name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::basename_from_path;

    #[test]
    fn basename_from_path_handles_backslash() {
        let name = basename_from_path("C:\\Program Files\\Foo\\Bar.exe");
        assert_eq!(name.as_deref(), Some("bar.exe"));
    }

    #[test]
    fn basename_from_path_handles_forwardslash() {
        let name = basename_from_path("C:/Foo/Bar.exe");
        assert_eq!(name.as_deref(), Some("bar.exe"));
    }
}
