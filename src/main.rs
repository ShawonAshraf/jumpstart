mod app_launcher;
mod config;

#[cfg(windows)]
mod monitor;

#[cfg(windows)]
mod window;

#[cfg(test)]
mod mock;

use app_launcher::launch_and_position_applications;
use config::load_config;
use tracing::{error, info};
use clap::Parser;

#[allow(clippy::single_component_path_imports)]
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "jumpstart")]
#[command(about = "Application launcher for positioning windows")]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yml")]
    config: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing subscriber with default info level
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting application launcher...");

    // Load configuration
    let config = load_config(&cli.config)?;
    info!(
        "Loaded configuration from '{}' with {} applications",
        cli.config,
        config.applications.len()
    );

    // Launch and position applications
    if let Err(e) = launch_and_position_applications(&config) {
        error!("Failed to launch and position applications: {}", e);
        return Err(e.into());
    }

    info!("Application launcher completed successfully");
    Ok(())
}
