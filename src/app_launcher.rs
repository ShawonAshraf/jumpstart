use std::process::Command;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use crate::config::{Config, Application};
use crate::monitor::{get_monitors, get_monitor_by_number, calculate_window_position};
use crate::window::{find_window_by_title, position_window};

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

pub fn launch_and_position_applications(config: &Config) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Side;

    #[test]
    fn test_launch_application_invalid_executable() {
        let result = launch_application("nonexistent_executable.exe");
        // This should fail since the executable doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_launch_and_position_applications_empty_config() {
        let empty_config = Config {
            applications: vec![],
        };
        
        let result = launch_and_position_applications(&empty_config);
        // This should succeed since there are no applications to launch
        assert!(result.is_ok());
    }

    #[test]
    fn test_launch_and_position_applications_invalid_display() {
        // Create a test config with an invalid display number
        let test_config = Config {
            applications: vec![
                Application {
                    name: "Test App".to_string(),
                    display: 999, // Invalid display number
                    side: Side::Left,
                    executable: "cmd.exe".to_string(), // Use a valid executable to avoid launch failure
                },
            ],
        };
        
        let result = launch_and_position_applications(&test_config);
        // The function should succeed even with invalid display number
        // It just logs an error and continues
        assert!(result.is_ok());
    }
}
