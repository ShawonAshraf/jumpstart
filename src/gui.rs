use crate::config::{load_config, load_default_config, get_default_config_content, Config};
use crate::app_launcher;
use eframe::egui::{self, Color32, RichText, Vec2};
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
    show_config_editor: bool,
    editor_content: String,
    theme: Theme,
}

#[derive(Debug, Clone, Copy)]
enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl JumpstartGui {
    pub fn new() -> Self {
        let mut gui = Self {
            config_path: "config.yml".to_string(),
            selected_config: None,
            config: None,
            is_running: false,
            status_message: "Welcome to Jumpstart! 👋\nLoad a configuration or create a new one to get started.".to_string(),
            operation_in_progress: false,
            show_config_editor: false,
            editor_content: get_default_config_content().to_string(),
            theme: Theme::Dark,
        };

        // Try to load default embedded config
        if let Ok(default_config) = load_default_config() {
            gui.config = Some(default_config);
            gui.status_message = "Loaded default configuration. You can edit it or load another file.".to_string();
        }

        gui
    }

    pub fn with_initial_config(config_path: String) -> Self {
        let mut gui = Self::new();
        gui.config_path = config_path.clone();

        // Try to load the initial config
        match load_config(&config_path) {
            Ok(config) => {
                gui.config = Some(config);
                gui.selected_config = Some(PathBuf::from(config_path.clone()));
                gui.status_message = format!("✅ Loaded configuration from {}", config_path);
            }
            Err(e) => {
                warn!("Failed to load initial config '{}': {}", config_path, e);
                gui.status_message = format!("⚠️ Failed to load config '{}': {}. Using default configuration.", config_path, e);
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
                    self.status_message = format!("✅ Loaded configuration from {}", path.display());
                    info!("Loaded config from: {}", path.display());
                }
                Err(e) => {
                    self.config = None;
                    self.status_message = format!("❌ Failed to load config: {}", e);
                    error!("Failed to load config from '{}': {}", path.display(), e);
                }
            }
        }
    }

    fn save_current_config(&mut self) {
        if let Some(ref path) = self.selected_config {
            match std::fs::write(path, &self.editor_content) {
                Ok(()) => {
                    self.status_message = format!("✅ Saved configuration to {}", path.display());
                    // Reload the config
                    match load_config(&self.config_path) {
                        Ok(config) => {
                            self.config = Some(config);
                        }
                        Err(e) => {
                            self.status_message = format!("⚠️ Saved but failed to reload: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("❌ Failed to save config: {}", e);
                }
            }
        } else {
            // Save as new file
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("YAML Files", &["yml", "yaml"])
                .set_file_name("config.yml")
                .save_file()
            {
                match std::fs::write(&path, &self.editor_content) {
                    Ok(()) => {
                        self.selected_config = Some(path.clone());
                        self.config_path = path.to_string_lossy().to_string();
                        self.status_message = format!("✅ Saved new configuration to {}", path.display());
                        // Reload the config
                        match load_config(&self.config_path) {
                            Ok(config) => {
                                self.config = Some(config);
                            }
                            Err(e) => {
                                self.status_message = format!("⚠️ Saved but failed to reload: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Failed to save config: {}", e);
                    }
                }
            }
        }
    }

    fn load_default_config_content(&mut self) {
        self.editor_content = get_default_config_content().to_string();
        self.status_message = "Loaded default configuration template".to_string();
    }

    fn start_applications(&mut self) {
        if self.config.is_none() {
            self.status_message = "❌ No configuration loaded. Please select a config file first.".to_string();
            return;
        }

        if self.is_running {
            self.status_message = "⏳ Applications are already being launched.".to_string();
            return;
        }

        self.is_running = true;
        self.status_message = "🚀 Starting applications...".to_string();

        // For now, run synchronously to avoid threading complications
        // The GUI will be responsive enough for this short operation
        let config = self.config.as_ref().unwrap().clone();

        match app_launcher::launch_and_position_applications(&config) {
            Ok(()) => {
                self.status_message = "✅ Applications launched successfully!".to_string();
                info!("All applications launched and positioned successfully");
            }
            Err(e) => {
                self.status_message = format!("❌ Error launching applications: {}", e);
                error!("Failed to launch and position applications: {}", e);
            }
        }

        self.is_running = false;
    }

    fn update_status(&mut self) {
        // Check if we should reset the running state
        if self.is_running && !self.operation_in_progress {
            self.is_running = false;
            if self.status_message.starts_with("🚀 Starting") {
                self.status_message = "✅ Application launching completed. Check logs for details.".to_string();
            }
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        match self.theme {
            Theme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Dark => {
                let mut visuals = egui::Visuals::dark();
                visuals.window_fill = Color32::from_gray(24);
                visuals.panel_fill = Color32::from_gray(24);
                ctx.set_visuals(visuals);
            }
        }
    }

    fn render_config_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("⚙️ Configuration").size(16.0).color(self.get_accent_color()));
            ui.add_space(8.0);

            // Current config display
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📁 Current:").size(13.0));
                    ui.separator();
                    if let Some(ref path) = self.selected_config {
                        ui.label(RichText::new(path.display().to_string()).color(Color32::from_rgb(100, 200, 100)));
                    } else {
                        ui.label(RichText::new("Using embedded default").color(Color32::from_rgb(200, 200, 100)));
                    }
                });
            });

            ui.add_space(8.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("📂 Load").clicked() {
                    self.select_config_file();
                }

                if ui.button("📝 Edit").clicked() {
                    self.show_config_editor = !self.show_config_editor;
                    if self.show_config_editor {
                        // Load current config content for editing
                        if let Some(ref path) = self.selected_config {
                            match std::fs::read_to_string(path) {
                                Ok(content) => {
                                    self.editor_content = content;
                                }
                                Err(_) => {
                                    self.load_default_config_content();
                                }
                            }
                        } else {
                            self.load_default_config_content();
                        }
                    }
                }

                if ui.button("🔄 Reset").clicked() {
                    if let Ok(default_config) = load_default_config() {
                        self.config = Some(default_config);
                        self.selected_config = None;
                        self.status_message = "✅ Reset to default configuration".to_string();
                    }
                }
            });
        });
    }

    fn render_applications_preview(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("🚀 Applications").size(16.0).color(self.get_accent_color()));
            ui.add_space(8.0);

            if let Some(ref config) = self.config {
                if config.applications.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("No applications configured").color(Color32::from_rgb(150, 150, 150)));
                    });
                } else {
                    // Create a scrollable area for applications
                    // Show more applications with increased height
                    egui::ScrollArea::vertical()
                        .min_scrolled_height(480.0)
                        .show(ui, |ui| {
                            for (index, app) in config.applications.iter().enumerate() {
                                self.render_application_card(ui, app, index);
                            }
                        });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("No configuration loaded").color(Color32::from_rgb(150, 150, 150)));
                });
            }
        });
    }

    fn render_application_card(&self, ui: &mut egui::Ui, app: &crate::config::Application, index: usize) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Application icon/number
                ui.label(RichText::new(format!("{}. {}", index + 1, app.name))
                    .size(14.0)
                    .color(Color32::from_rgb(100, 150, 200)));

                ui.separator();

                // Display info
                let display_color = match app.display {
                    1 => Color32::from_rgb(100, 200, 100),
                    2 => Color32::from_rgb(200, 200, 100),
                    _ => Color32::from_rgb(200, 150, 100),
                };
                ui.label(RichText::new(format!("D{}", app.display))
                    .size(12.0)
                    .color(display_color));

                ui.separator();

                // Side info
                let side_color = match app.side {
                    crate::config::Side::Left => Color32::from_rgb(150, 150, 200),
                    crate::config::Side::Right => Color32::from_rgb(200, 150, 150),
                };
                ui.label(RichText::new(format!("{:?}", app.side))
                    .size(12.0)
                    .color(side_color));
            });

            // Show executable path on a smaller line
            ui.add_space(2.0);
            ui.label(RichText::new(format!("📄 {}", app.executable))
                .size(10.0)
                .color(Color32::from_rgb(120, 120, 120)));
        });
        ui.add_space(6.0);
    }

    fn render_controls(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("🎮 Controls").size(16.0).color(self.get_accent_color()));
            ui.add_space(8.0);

            // Start button with better styling
            let start_button_enabled = self.config.is_some() && !self.is_running;

            let start_text = if self.is_running {
                "⏳ Launching..."
            } else {
                "🚀 Launch"
            };

            let button_color = if start_button_enabled {
                Color32::from_rgb(52, 152, 219) // Blue
            } else {
                Color32::from_rgb(150, 150, 150) // Gray
            };

            if ui.add_enabled(
                start_button_enabled,
                egui::Button::new(RichText::new(start_text).size(15.0).color(Color32::WHITE))
                    .fill(button_color)
                    .min_size(Vec2::new(280.0, 36.0))
            ).clicked() {
                self.start_applications();
            }

            ui.add_space(12.0);

            // Status message with better styling
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📊 Status:").size(13.0).strong());
                });
                ui.add_space(2.0);
                ui.label(RichText::new(&self.status_message).size(12.0));
            });

            // Progress indicator
            if self.is_running {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Launching applications...").size(12.0));
                });
            }
        });
    }

    fn render_config_editor(&mut self, ctx: &egui::Context) {
        let mut keep_open = true;
        let window_title = if let Some(ref path) = self.selected_config {
            format!("Editing: {}", path.file_name().unwrap().to_str().unwrap_or("config.yml"))
        } else {
            "Editing: New Configuration".to_string()
        };

        egui::Window::new(window_title)
            .open(&mut keep_open)
            .default_size(Vec2::new(800.0, 600.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        self.save_current_config();
                    }
                    if ui.button("📄 Save As").clicked() {
                        self.selected_config = None; // This will trigger save dialog
                        self.save_current_config();
                    }
                    if ui.button("🔄 Reset to Default").clicked() {
                        self.load_default_config_content();
                    }
                    ui.separator();
                    ui.label("YAML Configuration:");
                });

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_salt("config_editor")
                    .show(ui, |ui| {
                        ui.add_sized(
                            [ui.available_width(), ui.available_height() - 20.0],
                            egui::TextEdit::multiline(&mut self.editor_content)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                        );
                    });
            });

        if !keep_open {
            self.show_config_editor = false;
        }
    }

    fn get_accent_color(&self) -> Color32 {
        match self.theme {
            Theme::Light => Color32::from_rgb(52, 152, 219),
            Theme::Dark => Color32::from_rgb(100, 200, 200),
        }
    }
}

impl eframe::App for JumpstartGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.apply_theme(ctx);

        // Update internal state
        self.update_status();

        // Show config editor if open
        if self.show_config_editor {
            self.render_config_editor(ctx);
        }

        // Main layout with compact design and proper padding
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add padding around entire content
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(24.0, 16.0))
                .show(ui, |ui| {
                    // Header with theme switcher in top right
                    ui.horizontal(|ui| {
                        // Left side - Title and description
                        ui.vertical(|ui| {
                            ui.heading(RichText::new("🚀 Jumpstart Application Launcher")
                                .size(20.0)
                                .color(self.get_accent_color()));
                            ui.label(RichText::new("Automatically launch and position your applications")
                                .color(Color32::from_rgb(150, 150, 150)));
                        });

                        // Right side - Theme switcher
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(match self.theme {
                                Theme::Light => "🌙 Dark",
                                Theme::Dark => "☀️ Light",
                            }).clicked() {
                                self.theme = match self.theme {
                                    Theme::Light => Theme::Dark,
                                    Theme::Dark => Theme::Light,
                                };
                            }

                            ui.label(RichText::new("v0.1.0").color(Color32::from_rgb(120, 120, 120)));
                        });
                    });

                    ui.add_space(16.0);

                    // Main content with compact grid layout (2 columns)
                    egui::Grid::new("main_layout")
                        .num_columns(2)
                        .spacing([16.0, 16.0])
                        .show(ui, |ui| {

                            // Left column - Configuration and Applications
                            ui.vertical(|ui| {
                                ui.set_width(450.0);
                                self.render_config_panel(ui);
                                ui.add_space(16.0);
                                self.render_applications_preview(ui);
                            });

                            // Right column - Controls
                            ui.vertical(|ui| {
                                ui.set_width(300.0);
                                ui.set_min_height(350.0);
                                self.render_controls(ui);
                            });

                            ui.end_row();
                        });
                });
        });
    }
}
