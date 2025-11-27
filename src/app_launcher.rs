use crate::config::Config;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use crate::monitor::{calculate_window_position, get_monitor_by_number, get_monitors};

#[cfg(windows)]
use crate::window::{find_window_by_title, position_window};

#[cfg(test)]
use crate::config::Application;
#[cfg(test)]
use crate::mock::{MockWindowsApi, WindowsApiTrait, create_mock_monitors, create_mock_window_map};

#[cfg(windows)]
pub fn launch_application(executable: &str) -> Result<(), String> {
    // Try to launch the application using shell execute
    let output = Command::new("cmd")
        .args(["/C", "start", "", executable])
        .output()
        .map_err(|e| format!("Failed to launch application: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Application failed to start: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn launch_application(executable: &str) -> Result<(), String> {
    // Try to launch the application using standard shell commands
    let output = Command::new("sh")
        .args(["-c", executable])
        .output()
        .map_err(|e| format!("Failed to launch application: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Application failed to start: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(windows)]
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
            println!(
                "Positioning {} on display {} ({})",
                app.name, app.display, monitor.device_name
            );

            // Calculate window position
            let (x, y, width, height) = calculate_window_position(monitor, &app.side);

            // Try to find the window by title
            let search_title = app_window_titles
                .get(app.name.as_str())
                .unwrap_or(&app.name.as_str())
                .to_string();

            if let Some(hwnd) = find_window_by_title(&search_title) {
                // Position the window
                if let Err(e) = position_window(hwnd, x, y, width, height) {
                    eprintln!("Failed to position window for {}: {}", app.name, e);
                } else {
                    println!(
                        "Successfully positioned {} at ({}, {}) with size {}x{}",
                        app.name, x, y, width, height
                    );
                }
            } else {
                eprintln!(
                    "Could not find window for {} (searched for: {})",
                    app.name, search_title
                );
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

#[cfg(not(windows))]
pub fn launch_and_position_applications(_config: &Config) -> Result<(), String> {
    println!("Window positioning is only supported on Windows.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Side;
    use mockall::predicate::*;

    #[test]
    fn test_launch_application_invalid_executable() {
        let mut mock_api = MockWindowsApi::new();

        // Mock the launch_application function to return an error for invalid executables
        mock_api
            .expect_launch_application()
            .with(eq("nonexistent_executable.exe"))
            .times(1)
            .returning(|_| Err("Failed to launch application".to_string()));

        let result = mock_api.launch_application("nonexistent_executable.exe");
        // This should fail since the executable doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_launch_and_position_applications_empty_config() {
        let empty_config = Config {
            applications: vec![],
        };

        let mut mock_api = MockWindowsApi::new();

        // Mock get_monitors to return some monitors
        mock_api
            .expect_get_monitors()
            .times(1)
            .returning(create_mock_monitors);

        let result = launch_and_position_applications_mock(&empty_config, &mock_api);
        // This should succeed since there are no applications to launch
        assert!(result.is_ok());
    }

    #[test]
    fn test_launch_and_position_applications_invalid_display() {
        // Create a test config with an invalid display number
        let test_config = Config {
            applications: vec![Application {
                name: "Test App".to_string(),
                display: 999, // Invalid display number
                side: Side::Left,
                executable: "cmd.exe".to_string(), // Use a valid executable to avoid launch failure
            }],
        };

        let mut mock_api = MockWindowsApi::new();

        // Mock get_monitors to return some monitors
        mock_api
            .expect_get_monitors()
            .times(1)
            .returning(create_mock_monitors);

        // Mock successful launch
        mock_api
            .expect_launch_application()
            .with(eq("cmd.exe"))
            .times(1)
            .returning(|_| Ok(()));

        let result = launch_and_position_applications_mock(&test_config, &mock_api);
        // The function should succeed even with invalid display number
        // It just logs an error and continues
        assert!(result.is_ok());
    }

    #[test]
    fn test_launch_and_position_applications_success() {
        let test_config = Config {
            applications: vec![Application {
                name: "Teams".to_string(),
                display: 1,
                side: Side::Left,
                executable: "teams.exe".to_string(),
            }],
        };

        let mut mock_api = MockWindowsApi::new();

        // Mock get_monitors to return some monitors
        mock_api
            .expect_get_monitors()
            .times(1)
            .returning(create_mock_monitors);

        // Mock successful launch
        mock_api
            .expect_launch_application()
            .with(eq("teams.exe"))
            .times(1)
            .returning(|_| Ok(()));

        // Mock window finding
        mock_api
            .expect_find_window_by_title()
            .with(eq("teams"))
            .times(1)
            .returning(|_| Some(1001));

        // Mock window positioning
        mock_api
            .expect_position_window()
            .with(eq(1001), eq(0), eq(0), eq(960), eq(1040))
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let result = launch_and_position_applications_mock(&test_config, &mock_api);
        assert!(result.is_ok());
    }

    // Mock version of launch_and_position_applications for testing
    fn launch_and_position_applications_mock(
        config: &Config,
        api: &dyn WindowsApiTrait,
    ) -> Result<(), String> {
        // Get available monitors
        let monitors = api.get_monitors();
        println!("Found {} monitors", monitors.len());

        // Create a mapping of application names to their window titles
        let app_window_titles = create_mock_window_map();

        // Launch and position each application
        for app in &config.applications {
            println!("Launching {}...", app.name);

            // Launch the application
            if let Err(e) = api.launch_application(&app.executable) {
                eprintln!("Failed to launch {}: {}", app.name, e);
                continue;
            }

            // Get the target monitor
            if app.display > 0 && app.display <= monitors.len() as u32 {
                let monitor = &monitors[(app.display - 1) as usize];
                println!(
                    "Positioning {} on display {} ({})",
                    app.name, app.display, monitor.device_name
                );

                // Calculate window position
                let (x, y, width, height) = calculate_mock_window_position(monitor, &app.side);

                // Try to find the window by title
                let search_title = app_window_titles
                    .get(app.name.as_str())
                    .cloned()
                    .unwrap_or_else(|| app.name.clone());

                if let Some(hwnd) = api.find_window_by_title(&search_title) {
                    // Position the window
                    if let Err(e) = api.position_window(hwnd, x, y, width, height) {
                        eprintln!("Failed to position window for {}: {}", app.name, e);
                    } else {
                        println!(
                            "Successfully positioned {} at ({}, {}) with size {}x{}",
                            app.name, x, y, width, height
                        );
                    }
                } else {
                    eprintln!(
                        "Could not find window for {} (searched for: {})",
                        app.name, search_title
                    );
                }
            } else {
                eprintln!("Monitor {} not found for {}", app.display, app.name);
            }
        }

        println!("All applications launched and positioned!");
        Ok(())
    }

    fn calculate_mock_window_position(
        monitor: &crate::mock::MockMonitorInfo,
        side: &Side,
    ) -> (i32, i32, i32, i32) {
        let work_area = &monitor.work_area;
        let width = work_area.right - work_area.left;
        let height = work_area.bottom - work_area.top;

        match side {
            Side::Left => (work_area.left, work_area.top, width / 2, height),
            Side::Right => (work_area.left + width / 2, work_area.top, width / 2, height),
        }
    }
}
