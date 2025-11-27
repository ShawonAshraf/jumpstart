#[cfg(test)]
use mockall::{mock, predicate::*};
use std::collections::HashMap;

// Mock structures for testing
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MockMonitorInfo {
    pub handle: usize,
    pub rect: MockRect,
    pub work_area: MockRect,
    pub device_name: String,
}

#[derive(Debug, Clone)]
pub struct MockRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

// Trait for Windows API operations
pub trait WindowsApiTrait {
    fn get_monitors(&self) -> Vec<MockMonitorInfo>;
    fn find_window_by_title(&self, partial_title: &str) -> Option<usize>;
    fn position_window(
        &self,
        hwnd: usize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String>;
    fn launch_application(&self, executable: &str) -> Result<(), String>;
}

// Mock implementations for Windows API functions
#[cfg(test)]
mock! {
    pub WindowsApi {}

    impl WindowsApiTrait for WindowsApi {
        fn get_monitors(&self) -> Vec<MockMonitorInfo>;
        fn find_window_by_title(&self, partial_title: &str) -> Option<usize>;
        fn position_window(&self, hwnd: usize, x: i32, y: i32, width: i32, height: i32) -> Result<(), String>;
        fn launch_application(&self, executable: &str) -> Result<(), String>;
    }
}

#[cfg(test)]
pub fn create_mock_monitors() -> Vec<MockMonitorInfo> {
    vec![
        MockMonitorInfo {
            handle: 1,
            rect: MockRect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            work_area: MockRect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
            },
            device_name: "Monitor1".to_string(),
        },
        MockMonitorInfo {
            handle: 2,
            rect: MockRect {
                left: 1920,
                top: 0,
                right: 3840,
                bottom: 1080,
            },
            work_area: MockRect {
                left: 1920,
                top: 0,
                right: 3840,
                bottom: 1040,
            },
            device_name: "Monitor2".to_string(),
        },
    ]
}

#[cfg(test)]
pub fn create_mock_window_map() -> HashMap<String, String> {
    let mut window_map = HashMap::new();
    window_map.insert("Teams".to_string(), "teams".to_string());
    window_map.insert("Outlook".to_string(), "outlook".to_string());
    window_map.insert("Slack".to_string(), "slack".to_string());
    window_map.insert("Notion".to_string(), "notion".to_string());
    window_map
}
