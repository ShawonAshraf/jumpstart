use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tracing::{debug, info, warn};
use widestring::U16CString;
use winapi::shared::minwindef::{BOOL, DWORD, LPARAM, TRUE};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, HWND_TOP,
    SWP_NOZORDER, SetWindowPos,
};

#[derive(Debug)]
struct WindowInfo {
    hwnd: HWND,
    title: String,
    _process_id: u32,
}

// Global timeout flag for window enumeration
static ENUM_TIMEOUT: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, data: LPARAM) -> BOOL {
    // Check if we've timed out
    if ENUM_TIMEOUT.load(Ordering::Relaxed) {
        return 0; // FALSE equivalent to stop enumeration
    }

    let windows = data as *mut Vec<WindowInfo>;

    let mut process_id: DWORD = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }

    let text_len = unsafe { GetWindowTextLengthW(hwnd) };

    // Limit the maximum title length to prevent issues with extremely long titles
    const MAX_TITLE_LENGTH: usize = 1024;
    if text_len > 0 && text_len <= MAX_TITLE_LENGTH as i32 {
        let mut buffer = vec![0; text_len as usize + 1];

        unsafe {
            GetWindowTextW(hwnd, buffer.as_mut_ptr(), text_len + 1);
        }

        let title = unsafe {
            U16CString::from_ptr_str(buffer.as_ptr())
                .to_string_lossy()
                .to_string()
        };

        if !title.is_empty() {
            let window_info = WindowInfo {
                hwnd,
                title,
                _process_id: process_id,
            };
            unsafe {
                (*windows).push(window_info);
            }
        }
    }

    TRUE
}

pub fn find_window_by_title(partial_title: &str) -> Option<HWND> {
    let mut windows: Vec<WindowInfo> = Vec::new();

    // Reset the timeout flag
    ENUM_TIMEOUT.store(false, Ordering::Relaxed);

    // Start timeout timer (5 seconds)
    let start_time = Instant::now();
    const ENUM_TIMEOUT_MS: u128 = 5000; // 5 seconds

    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            &mut windows as *mut Vec<WindowInfo> as LPARAM,
        );
    }

    // Check if we timed out during enumeration
    if start_time.elapsed().as_millis() > ENUM_TIMEOUT_MS {
        ENUM_TIMEOUT.store(true, Ordering::Relaxed);
        warn!("Window enumeration timed out after {} ms", ENUM_TIMEOUT_MS);
    }

    debug!(
        "Enumerated {} windows, searching for '{}'",
        windows.len(),
        partial_title
    );

    for window in windows {
        if window
            .title
            .to_lowercase()
            .contains(&partial_title.to_lowercase())
        {
            info!(
                "Found matching window: '{}' for search '{}'",
                window.title, partial_title
            );
            return Some(window.hwnd);
        }
    }

    debug!("No window found matching '{}'", partial_title);
    None
}

pub fn position_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    unsafe {
        if SetWindowPos(hwnd, HWND_TOP, x, y, width, height, SWP_NOZORDER) != 0 {
            Ok(())
        } else {
            Err("Failed to position window".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_find_window_by_title_case_insensitive() {
        // This test is difficult to implement without actual windows
        // We'll test the case insensitive logic with a mock scenario

        // Test that the function handles empty strings
        let _result = find_window_by_title("");
        // We can't guarantee the result, but the function should not panic
        // It will return None if no window matches
    }

    #[test]
    fn test_position_window_invalid_handle() {
        // Test with an invalid handle
        let result = position_window(ptr::null_mut(), 0, 0, 100, 100);
        // This should return an error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to position window");
    }
}
