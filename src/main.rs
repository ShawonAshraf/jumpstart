mod app_launcher;
mod config;
mod monitor;
mod window;

#[cfg(test)]
mod mock;

use app_launcher::launch_and_position_applications;
use config::load_config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting application launcher...");

    // Load configuration
    let config = load_config()?;
    println!(
        "Loaded configuration with {} applications",
        config.applications.len()
    );

    // Launch and position applications
    launch_and_position_applications(&config)?;

    Ok(())
}
