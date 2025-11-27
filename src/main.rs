use serde::{Deserialize, Serialize, Deserializer};
use std::process::Command;
use std::thread;
use std::time::Duration;
use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, SetWindowPos, HWND_TOP, SWP_NOZORDER, EnumWindows, GetWindowThreadProcessId, GetWindowTextLengthW, GetWindowTextW};
use winapi::shared::windef::{HMONITOR, HWND, RECT, LPRECT, HDC};
use winapi::shared::minwindef::{DWORD, LPARAM, BOOL, TRUE};
use widestring::U16CString;
use std::ptr;
use std::mem;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub enum Side {
    Left,
    Right,
}

impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "left" => Ok(Side::Left),
            "right" => Ok(Side::Right),
            _ => Err(serde::de::Error::custom(format!("Invalid side: {}", s))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub display: u32,
    pub side: Side,
    pub executable: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub applications: Vec<Application>,
}

#[derive(Clone)]
pub struct MonitorInfo {
    pub handle: HMONITOR,
    pub rect: RECT,
    pub work_area: RECT,
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

pub fn calculate_window_position(monitor: &MonitorInfo, side: &Side) -> (i32, i32, i32, i32) {
    let work_area = &monitor.work_area;
    let width = work_area.right - work_area.left;
    let height = work_area.bottom - work_area.top;
    
    match side {
        Side::Left => (
            work_area.left,
            work_area.top,
            width / 2,
            height,
        ),
        Side::Right => (
            work_area.left + width / 2,
            work_area.top,
            width / 2,
            height,
        ),
    }
}

#[derive(Debug)]
struct WindowInfo {
    hwnd: HWND,
    title: String,
    _process_id: u32,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, data: LPARAM) -> BOOL {
    let windows = data as *mut Vec<WindowInfo>;
    
    let mut process_id: DWORD = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }
    
    let text_len = unsafe { GetWindowTextLengthW(hwnd) };
    if text_len > 0 {
        let mut buffer: Vec<u16> = Vec::with_capacity(text_len as usize + 1);
        buffer.resize(text_len as usize + 1, 0);
        
        unsafe {
            GetWindowTextW(hwnd, buffer.as_mut_ptr(), text_len + 1);
        }
        
        let title = unsafe {
            U16CString::from_ptr_str(buffer.as_ptr()).to_string_lossy().to_string()
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
    unsafe {
        EnumWindows(Some(enum_windows_proc), &mut windows as *mut Vec<WindowInfo> as LPARAM);
    }
    
    for window in windows {
        if window.title.to_lowercase().contains(&partial_title.to_lowercase()) {
            return Some(window.hwnd);
        }
    }
    
    None
}

pub fn position_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    unsafe {
        if SetWindowPos(
            hwnd,
            HWND_TOP,
            x,
            y,
            width,
            height,
            SWP_NOZORDER,
        ) != 0
        {
            Ok(())
        } else {
            Err("Failed to position window".to_string())
        }
    }
}

pub fn launch_application(executable: &str) -> Result<(), String> {
    // Try to launch the application using shell execute
    let output = Command::new("cmd")
        .args(&["/C", "start", "", executable])
        .output()
        .map_err(|e| format!("Failed to launch application: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Application failed to start: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

pub fn load_config() -> Result<Config, String> {
    let yaml_content = std::fs::read_to_string("config.yml")
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    serde_yaml::from_str(&yaml_content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting application launcher...");
    
    // Load configuration
    let config = load_config()?;
    println!("Loaded configuration with {} applications", config.applications.len());
    
    // Get available monitors
    let monitors = get_monitors();
    println!("Found {} monitors", monitors.len());
    
    // Create a mapping of application names to their window titles
    let mut app_window_titles = HashMap::new();
    app_window_titles.insert("Teams", "teams");
    app_window_titles.insert("Outlook", "outlook");
    app_window_titles.insert("Slack", "slack");
    app_window_titles.insert("Notion", "notion");
    
    // Launch and position each application
    for app in &config.applications {
        println!("Launching {}...", app.name);
        
        // Launch the application
        if let Err(e) = launch_application(&app.executable) {
            eprintln!("Failed to launch {}: {}", app.name, e);
            continue;
        }
        
        // Wait for the application to start and create its window
        thread::sleep(Duration::from_secs(5));
        
        // Get the target monitor
        if let Some(monitor) = get_monitor_by_number(&monitors, app.display) {
            println!("Positioning {} on display {} ({})", app.name, app.display, monitor.device_name);
            
            // Calculate window position
            let (x, y, width, height) = calculate_window_position(monitor, &app.side);
            
            // Try to find the window by title
            let search_title = app_window_titles.get(app.name.as_str()).unwrap_or(&app.name.as_str()).to_string();
            
            if let Some(hwnd) = find_window_by_title(&search_title) {
                // Position the window
                if let Err(e) = position_window(hwnd, x, y, width, height) {
                    eprintln!("Failed to position window for {}: {}", app.name, e);
                } else {
                    println!("Successfully positioned {} at ({}, {}) with size {}x{}", 
                            app.name, x, y, width, height);
                }
            } else {
                eprintln!("Could not find window for {} (searched for: {})", app.name, search_title);
            }
        } else {
            eprintln!("Monitor {} not found for {}", app.display, app.name);
        }
        
        // Wait a bit before launching the next application
        thread::sleep(Duration::from_secs(2));
    }
    
    println!("All applications launched and positioned!");
    Ok(())
}
