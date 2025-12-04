use crate::config::{load_config, Config};
use crate::app_launcher;
use eframe::egui;
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Default)]
pub struct JumpstartGui {
    config_path: String,
    selected_config: Option<PathBuf>,
    config: Option<Config>,
    is_running: bool,
    status_message: String,
    operation_in_progress: bool,
}

impl JumpstartGui {
    pub fn new() -> Self {
        Self {
            config_path: "config.yml".to_string(),
            selected_config: None,
            config: None,
            is_running: false,
            status_message: "Ready. Please select a configuration file.".to_string(),
            operation_in_progress: false,
        }
    }

    pub fn with_initial_config(config_path: String) -> Self {
        let mut gui = Self::new();
        gui.config_path = config_path.clone();

        // Try to load the initial config
        match load_config(&config_path) {
            Ok(config) => {
                gui.config = Some(config);
                gui.selected_config = Some(PathBuf::from(config_path.clone()));
                gui.status_message = format!("Loaded configuration from {}", config_path);
            }
            Err(e) => {
                warn!("Failed to load initial config '{}': {}", config_path, e);
                gui.status_message = format!("Failed to load config '{}': {}", config_path, e);
            }
        }
        gui
    }

    fn select_config_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("YAML Files", &["yml", "yaml"])
            .set_directory(".")
            .pick_file()
        {
            self.selected_config = Some(path.clone());
            self.config_path = path.to_string_lossy().to_string();

            // Try to load the config
            match load_config(&self.config_path) {
                Ok(config) => {
                    self.config = Some(config);
                    self.status_message = format!("Loaded configuration from {}", path.display());
                    info!("Loaded config from: {}", path.display());
                }
                Err(e) => {
                    self.config = None;
                    self.status_message = format!("Failed to load config: {}", e);
                    error!("Failed to load config from '{}': {}", path.display(), e);
                }
            }
        }
    }

    fn start_applications(&mut self) {
        if self.config.is_none() {
            self.status_message = "No configuration loaded. Please select a config file first.".to_string();
            return;
        }

        if self.is_running {
            self.status_message = "Applications are already being launched.".to_string();
            return;
        }

        self.is_running = true;
        self.status_message = "Starting applications...".to_string();

        // For now, run synchronously to avoid threading complications
        // The GUI will be responsive enough for this short operation
        let config = self.config.as_ref().unwrap().clone();

        match app_launcher::launch_and_position_applications(&config) {
            Ok(()) => {
                self.status_message = "Applications launched successfully!".to_string();
                info!("All applications launched and positioned successfully");
            }
            Err(e) => {
                self.status_message = format!("Error launching applications: {}", e);
                error!("Failed to launch and position applications: {}", e);
            }
        }

        self.is_running = false;
    }

    fn update_status(&mut self) {
        // Check if we should reset the running state
        if self.is_running && !self.operation_in_progress {
            self.is_running = false;
            if self.status_message.starts_with("Starting applications") {
                self.status_message = "Application launching completed. Check logs for details.".to_string();
            }
        }
    }
}

impl eframe::App for JumpstartGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update internal state
        self.update_status();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Jumpstart Application Launcher");
            ui.add_space(20.0);

            // Config file selection section
            ui.group(|ui| {
                ui.heading("Configuration File");
                ui.add_space(10.0);

                // Display current config path
                if let Some(ref path) = self.selected_config {
                    ui.label(format!("Selected: {}", path.display()));
                } else {
                    ui.label("No configuration file selected");
                }

                ui.add_space(10.0);

                // File selection button
                if ui.button("Select Config File").clicked() {
                    self.select_config_file();
                }
            });

            ui.add_space(20.0);

            // Applications preview section
            if let Some(ref config) = self.config {
                ui.group(|ui| {
                    ui.heading("Applications to Launch");
                    ui.add_space(10.0);

                    for app in &config.applications {
                        ui.horizontal(|ui| {
                            ui.label(&app.name);
                            ui.separator();
                            ui.label(format!("Display: {}", app.display));
                            ui.separator();
                            ui.label(format!("Side: {:?}", app.side));
                        });
                    }

                    if config.applications.is_empty() {
                        ui.label("No applications configured");
                    }
                });
            }

            ui.add_space(20.0);

            // Control section
            ui.group(|ui| {
                ui.heading("Controls");
                ui.add_space(10.0);

                // Start button
                let start_button_enabled = self.config.is_some() && !self.is_running;
                if ui.add_enabled(start_button_enabled, egui::Button::new("Start"))
                    .clicked()
                {
                    self.start_applications();
                }

                ui.add_space(10.0);

                // Status message
                ui.group(|ui| {
                    ui.label("Status:");
                    ui.label(&self.status_message);
                });
            });

            ui.add_space(20.0);

            // Progress indicator
            if self.is_running {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Launching applications...");
                });
            }
        });
    }
}