use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW};
use winapi::shared::windef::{HMONITOR, HDC, LPRECT};
use winapi::shared::minwindef::{DWORD, LPARAM, BOOL, TRUE};
use widestring::U16CString;
use std::ptr;
use std::mem;

#[derive(Clone)]
pub struct MonitorInfo {
    pub handle: HMONITOR,
    pub rect: winapi::shared::windef::RECT,
    pub work_area: winapi::shared::windef::RECT,
    pub device_name: String,
}

impl std::fmt::Debug for MonitorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MonitorInfo")
            .field("handle", &self.handle)
            .field("rect", &format!("RECT {{ left: {}, top: {}, right: {}, bottom: {} }}", 
                                   self.rect.left, self.rect.top, self.rect.right, self.rect.bottom))
            .field("work_area", &format!("RECT {{ left: {}, top: {}, right: {}, bottom: {} }}", 
                                        self.work_area.left, self.work_area.top, self.work_area.right, self.work_area.bottom))
            .field("device_name", &self.device_name)
            .finish()
    }
}

unsafe extern "system" fn monitor_enum_proc(hmonitor: HMONITOR, _hdc: HDC, _rect: LPRECT, data: LPARAM) -> BOOL {
    let monitors = data as *mut Vec<MonitorInfo>;
    
    let mut monitor_info: MONITORINFOEXW = unsafe { mem::zeroed() };
    monitor_info.cbSize = mem::size_of::<MONITORINFOEXW>() as DWORD;
    
    if unsafe { GetMonitorInfoW(hmonitor, &mut monitor_info as *mut MONITORINFOEXW as *mut winapi::um::winuser::MONITORINFO) != 0 } {
        let device_name = unsafe {
            U16CString::from_ptr_str(monitor_info.szDevice.as_ptr()).to_string_lossy().to_string()
        };
        
        let monitor = MonitorInfo {
            handle: hmonitor,
            rect: monitor_info.rcMonitor,
            work_area: monitor_info.rcWork,
            device_name,
        };
        
        unsafe {
            (*monitors).push(monitor);
        }
    }
    
    TRUE
}

pub fn get_monitors() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null(),
            Some(monitor_enum_proc),
            &mut monitors as *mut Vec<MonitorInfo> as LPARAM,
        );
    }
    monitors
}

pub fn get_monitor_by_number(monitors: &[MonitorInfo], number: u32) -> Option<&MonitorInfo> {
    if number == 0 || number > monitors.len() as u32 {
        return None;
    }
    monitors.get((number - 1) as usize)
}

pub fn calculate_window_position(monitor: &MonitorInfo, side: &crate::config::Side) -> (i32, i32, i32, i32) {
    let work_area = &monitor.work_area;
    let width = work_area.right - work_area.left;
    let height = work_area.bottom - work_area.top;
    
    match side {
        crate::config::Side::Left => (
            work_area.left,
            work_area.top,
            width / 2,
            height,
        ),
        crate::config::Side::Right => (
            work_area.left + width / 2,
            work_area.top,
            width / 2,
            height,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_monitor_by_number() {
        let monitors = vec![
            MonitorInfo {
                handle: ptr::null_mut(),
                rect: winapi::shared::windef::RECT { left: 0, top: 0, right: 1920, bottom: 1080 },
                work_area: winapi::shared::windef::RECT { left: 0, top: 0, right: 1920, bottom: 1040 },
                device_name: "Monitor1".to_string(),
            },
            MonitorInfo {
                handle: ptr::null_mut(),
                rect: winapi::shared::windef::RECT { left: 1920, top: 0, right: 3840, bottom: 1080 },
                work_area: winapi::shared::windef::RECT { left: 1920, top: 0, right: 3840, bottom: 1040 },
                device_name: "Monitor2".to_string(),
            },
        ];

        // Test valid monitor numbers
        assert_eq!(get_monitor_by_number(&monitors, 1).unwrap().device_name, "Monitor1");
        assert_eq!(get_monitor_by_number(&monitors, 2).unwrap().device_name, "Monitor2");

        // Test invalid monitor numbers
        assert!(get_monitor_by_number(&monitors, 0).is_none());
        assert!(get_monitor_by_number(&monitors, 3).is_none());
    }

    #[test]
    fn test_calculate_window_position() {
        let monitor = MonitorInfo {
            handle: ptr::null_mut(),
            rect: winapi::shared::windef::RECT { left: 0, top: 0, right: 1920, bottom: 1080 },
            work_area: winapi::shared::windef::RECT { left: 0, top: 0, right: 1920, bottom: 1040 },
            device_name: "Test Monitor".to_string(),
        };

        // Test left side positioning
        let (x, y, width, height) = calculate_window_position(&monitor, &crate::config::Side::Left);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(width, 960);
        assert_eq!(height, 1040);

        // Test right side positioning
        let (x, y, width, height) = calculate_window_position(&monitor, &crate::config::Side::Right);
        assert_eq!(x, 960);
        assert_eq!(y, 0);
        assert_eq!(width, 960);
        assert_eq!(height, 1040);
    }
}
