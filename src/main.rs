mod app_launcher;
mod config;

#[cfg(windows)]
mod monitor;

#[cfg(windows)]
mod window;

#[cfg(test)]
mod mock;

mod gui;

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

    /// Launch in GUI mode instead of CLI mode
    #[arg(short, long, default_value_t = false)]
    gui: bool,
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

    if cli.gui {
        run_gui_mode(cli.config)?;
    } else {
        run_cli_mode(cli.config)?;
    }

    Ok(())
}

fn run_cli_mode(config_path: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting application launcher in CLI mode...");

    // Load configuration
    let config = load_config(&config_path)?;
    info!(
        "Loaded configuration from '{}' with {} applications",
        config_path,
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

fn run_gui_mode(config_path: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting application launcher in GUI mode...");

    // Initialize the GUI with the specified config path
    let app = gui::JumpstartGui::with_initial_config(config_path);

    // Set up the GUI options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_min_inner_size([500.0, 400.0])
            .with_title("Jumpstart Application Launcher"),
        ..Default::default()
    };

    // Run the GUI
    eframe::run_native(
        "Jumpstart",
        options,
        Box::new(|cc| {
            // Customize egui style here if needed
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app))
        }),
    ).map_err(|e| {
        error!("GUI error: {}", e);
        Box::<dyn std::error::Error>::from(e)
    })?;

    info!("GUI application closed");
    Ok(())
}
