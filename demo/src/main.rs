use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use aurora_egui::{App, CoverAction, Frame, NativeOptions};
use egui::RichText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Counter,
    EnvVars,
    Settings,
    Notifications,
    RuntimeDir,
}

struct DemoApp {
    counter: Arc<AtomicI32>,
    name: String,
    current_page: Page,
    show_menu: bool,
    transparent_bg: bool,
    original_panel_fill: egui::Color32,
    env_filter: String,

    // Notifications page state
    notification_service: Option<aurora_services::NotificationService>,
    notif_title: String,
    notif_body: String,
    notif_urgency: u8,
    last_notification_id: Option<u32>,
    notif_status: String,
    notif_capabilities: String,
    notif_server_info: String,

    // Runtime dir page state
    runtime_dir_root: String,
    runtime_dir_current: String,
    runtime_dir_entries: Vec<(String, bool)>,
    runtime_dir_filter: String,
}

impl App for DemoApp {
    fn update(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
        egui::Panel::top("top_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("☰").size(32.)).clicked() {
                    self.show_menu = !self.show_menu;
                }
                ui.heading(match self.current_page {
                    Page::Counter => "Counter",
                    Page::EnvVars => "Env Variables",
                    Page::Settings => "Settings",
                    Page::Notifications => "Notifications",
                    Page::RuntimeDir => "Runtime Dir",
                });
            });
        });

        if self.show_menu {
            egui::Panel::left("side_menu")
                .default_size(90.0)
                .show_inside(ui, |ui| {
                    ui.vertical(|ui| {
                        if ui
                            .selectable_label(self.current_page == Page::Counter, "Counter")
                            .clicked()
                        {
                            self.current_page = Page::Counter;
                            self.show_menu = false;
                        }
                        if ui
                            .selectable_label(self.current_page == Page::EnvVars, "Env Vars")
                            .clicked()
                        {
                            self.current_page = Page::EnvVars;
                            self.show_menu = false;
                        }
                        if ui
                            .selectable_label(self.current_page == Page::Settings, "Settings")
                            .clicked()
                        {
                            self.current_page = Page::Settings;
                            self.show_menu = false;
                        }
                        if ui
                            .selectable_label(
                                self.current_page == Page::Notifications,
                                "Notifications",
                            )
                            .clicked()
                        {
                            self.current_page = Page::Notifications;
                            self.show_menu = false;
                        }
                        if ui
                            .selectable_label(self.current_page == Page::RuntimeDir, "Runtime Dir")
                            .clicked()
                        {
                            self.current_page = Page::RuntimeDir;
                            self.show_menu = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| match self.current_page {
            Page::Counter => self.show_counter_page(ui),
            Page::EnvVars => self.show_env_vars_page(ui),
            Page::Settings => self.show_settings_page(ui, frame),
            Page::Notifications => self.show_notifications_page(ui),
            Page::RuntimeDir => self.show_runtime_dir_page(ui),
        });
    }

    fn cover_ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.heading(RichText::new("Aurora egui Demo").size(40.));
                ui.add_space(20.0);
                ui.label(
                    RichText::new(format!("Counter: {}", self.counter.load(Ordering::Relaxed)))
                        .size(32.),
                );
                ui.heading(format!(
                    "Current pixel ratio: {}",
                    ui.ctx().pixels_per_point()
                ));
            });
        });
    }

    fn cover_actions(&self) -> Vec<CoverAction> {
        vec![
            // CoverAction {
            //     id: "reset".to_string(),
            //     icon: "image://theme/icon-cover-previous".to_string(),
            // },
            // CoverAction {
            //     id: "add".to_string(),
            //     icon: "image://theme/icon-cover-next".to_string(),
            // },
        ]
    }
}

impl DemoApp {
    fn show_counter_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(format!("Hello, {}!", self.name));
            ui.add_space(20.0);

            ui.heading(format!("Counter: {}", self.counter.load(Ordering::Relaxed)));
            ui.add_space(10.0);

            ui.heading(format!(
                "Current pixel ratio: {}",
                ui.ctx().pixels_per_point()
            ));

            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    self.counter.fetch_sub(1, Ordering::Relaxed);
                }
                if ui.button("+").clicked() {
                    self.counter.fetch_add(1, Ordering::Relaxed);
                }
            });

            ui.add_space(20.0);
            ui.label("The cover page will show this counter when the app is backgrounded.");
        });
    }

    fn show_env_vars_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Environment Variables");
        ui.separator();

        let mut vars: Vec<(String, String)> = std::env::vars().collect();
        vars.sort_by(|a, b| a.0.cmp(&b.0));

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.env_filter);
                    ui.label(format!("Total: {}", vars.len()));
                });
                ui.separator();

                let filter = self.env_filter.to_lowercase();
                vars.retain(|(k, v)| {
                    k.to_lowercase().contains(&filter) || v.to_lowercase().contains(&filter)
                });

                egui::Grid::new("env_vars_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        for (key, value) in vars {
                            ui.add(egui::Label::new(
                                egui::RichText::new(key).monospace().strong(),
                            ));
                            ui.add(egui::Label::new(egui::RichText::new(value).monospace()));
                            ui.end_row();
                        }
                    });
            });
    }

    fn show_settings_page(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
        ui.heading("Settings");
        ui.separator();

        if ui.button("Toggle Status Bar").clicked() {
            frame.set_statusbar_visible(!frame.is_statusbar_visible());
        }
        if ui.button("Toggle Background").clicked() {
            self.transparent_bg = !self.transparent_bg;
            let new_fill = if self.transparent_bg {
                egui::Color32::TRANSPARENT
            } else {
                self.original_panel_fill
            };
            ui.ctx().global_style_mut(|style| {
                style.visuals.panel_fill = new_fill;
            });
        }
        if ui.button("Reset Counter").clicked() {
            self.counter.store(0, Ordering::Relaxed);
        }
    }

    fn show_notifications_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Notifications");
        ui.separator();

        // Lazy-init notification service
        if self.notification_service.is_none() {
            match aurora_services::NotificationService::new() {
                Ok(service) => {
                    self.notification_service = Some(service);
                    self.notif_status = "Service ready".to_string();
                }
                Err(e) => {
                    self.notif_status = format!("Failed to create service: {e}");
                }
            }
        }

        let Some(service) = self.notification_service.as_ref() else {
            ui.colored_label(ui.visuals().error_fg_color, &self.notif_status);
            return;
        };

        ui.group(|ui| {
            ui.label("Content");
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut self.notif_title);
            });
            ui.horizontal(|ui| {
                ui.label("Body: ");
                ui.text_edit_singleline(&mut self.notif_body);
            });
            ui.horizontal(|ui| {
                ui.label("Urgency:");
                ui.selectable_value(&mut self.notif_urgency, 0, "Low");
                ui.selectable_value(&mut self.notif_urgency, 1, "Normal");
                ui.selectable_value(&mut self.notif_urgency, 2, "Critical");
            });
        });

        ui.horizontal(|ui| {
            if ui.button("Show Notification").clicked() {
                let notif = aurora_services::NotificationBuilder::new()
                    .app_name("Aurora egui Demo")
                    .summary(&self.notif_title)
                    .body(&self.notif_body)
                    .urgency(self.notif_urgency)
                    .desktop_entry(
                        format!(
                            "{}.desktop",
                            aurora_services::package_info::app_id().unwrap()
                        )
                        .as_str(),
                    )
                    .build();

                match service.notify(&notif) {
                    Ok(id) => {
                        self.last_notification_id = Some(id);
                        self.notif_status = format!("Notification shown (id={id})");
                    }
                    Err(e) => {
                        self.notif_status = format!("Notify error: {e}");
                    }
                }
            }

            if ui
                .add_enabled(
                    self.last_notification_id.is_some(),
                    egui::Button::new("Close Last"),
                )
                .clicked()
            {
                if let Some(id) = self.last_notification_id {
                    match service.close(id) {
                        Ok(()) => {
                            self.notif_status = format!("Closed notification {id}");
                            self.last_notification_id = None;
                        }
                        Err(e) => {
                            self.notif_status = format!("Close error: {e}");
                        }
                    }
                }
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Get Capabilities").clicked() {
                match service.get_capabilities() {
                    Ok(caps) => {
                        self.notif_capabilities = caps.join(", ");
                        self.notif_status = "Capabilities retrieved".to_string();
                    }
                    Err(e) => {
                        self.notif_status = format!("Capabilities error: {e}");
                    }
                }
            }
            if ui.button("Get Server Info").clicked() {
                match service.get_server_info() {
                    Ok(info) => {
                        self.notif_server_info = format!(
                            "{} {} (vendor: {}, spec: {})",
                            info.name, info.version, info.vendor, info.spec_version
                        );
                        self.notif_status = "Server info retrieved".to_string();
                    }
                    Err(e) => {
                        self.notif_status = format!("Server info error: {e}");
                    }
                }
            }
        });

        ui.separator();

        if !self.notif_capabilities.is_empty() {
            ui.label(format!("Capabilities: {}", self.notif_capabilities));
        }
        if !self.notif_server_info.is_empty() {
            ui.label(format!("Server info: {}", self.notif_server_info));
        }
        if self.last_notification_id.is_some() {
            ui.label(format!(
                "Last notification ID: {}",
                self.last_notification_id.unwrap()
            ));
        }
        ui.colored_label(
            if self.notif_status.starts_with("Failed") || self.notif_status.contains("error") {
                ui.visuals().error_fg_color
            } else {
                ui.visuals().text_color()
            },
            format!("Status: {}", self.notif_status),
        );
    }

    fn show_runtime_dir_page(&mut self, ui: &mut egui::Ui) {
        // Lazy-load on first open or when path changes
        if self.runtime_dir_entries.is_empty() {
            self.runtime_dir_entries = read_dir_entries(&self.runtime_dir_current);
        }

        ui.heading("XDG_RUNTIME_DIR");
        ui.separator();

        // Breadcrumb / navigation bar
        ui.horizontal_wrapped(|ui| {
            ui.label("Path:");
            ui.monospace(&self.runtime_dir_current);
        });
        ui.horizontal(|ui| {
            if ui.button("🏠 Root").clicked() {
                self.runtime_dir_current = self.runtime_dir_root.clone();
                self.runtime_dir_entries = read_dir_entries(&self.runtime_dir_current);
            }
            if ui
                .add_enabled(
                    self.runtime_dir_current != self.runtime_dir_root,
                    egui::Button::new("⬆ Up"),
                )
                .clicked()
            {
                if let Some(parent) = std::path::Path::new(&self.runtime_dir_current).parent() {
                    self.runtime_dir_current = parent.to_string_lossy().to_string();
                    self.runtime_dir_entries = read_dir_entries(&self.runtime_dir_current);
                }
            }
            if ui.button("Refresh").clicked() {
                self.runtime_dir_entries = read_dir_entries(&self.runtime_dir_current);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.runtime_dir_filter);
            ui.label(format!("Total: {}", self.runtime_dir_entries.len()));
        });
        ui.separator();

        if self.runtime_dir_entries.is_empty() {
            ui.label("No entries (folder is empty or inaccessible).");
        } else {
            let filter = self.runtime_dir_filter.to_lowercase();
            let mut entries: Vec<(String, bool)> = self
                .runtime_dir_entries
                .iter()
                .filter(|(name, _)| name.to_lowercase().contains(&filter))
                .cloned()
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));

            let mut clicked_folder: Option<String> = None;

            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    egui::Grid::new("runtime_dir_grid")
                        .num_columns(1)
                        .striped(true)
                        .show(ui, |ui| {
                            for (name, is_dir) in &entries {
                                let label = ui
                                    .selectable_label(false, egui::RichText::new(name).monospace());
                                if label.clicked() && *is_dir {
                                    clicked_folder =
                                        Some(name.trim_start_matches("📁 ").to_string());
                                }
                                ui.end_row();
                            }
                        });
                });

            if let Some(folder) = clicked_folder {
                self.runtime_dir_current = format!("{}/{}", self.runtime_dir_current, folder);
                self.runtime_dir_entries = read_dir_entries(&self.runtime_dir_current);
            }
        }
    }
}

fn read_dir_entries(path: &str) -> Vec<(String, bool)> {
    let iter = match std::fs::read_dir(path) {
        Ok(iter) => iter,
        Err(_) => return Vec::new(),
    };
    iter.filter_map(|e| e.ok())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let file_type = e.file_type().ok();
            let is_dir = file_type.map_or(false, |ft| ft.is_dir());
            let prefix = if is_dir {
                "📁 "
            } else if file_type.map_or(false, |ft| ft.is_symlink()) {
                "🔗 "
            } else {
                "📄 "
            };
            (format!("{}{}", prefix, name), is_dir)
        })
        .collect()
}

fn main() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("com.lmaxyz.egui_aurora_app"),
        ..Default::default()
    };

    aurora_egui::run_native(
        "Aurora egui Demo",
        options,
        Box::new(|cc| {
            let original_panel_fill = cc.egui_ctx.global_style().visuals.panel_fill;
            let counter = Arc::new(AtomicI32::new(0));

            let counter_clone = counter.clone();
            let egui_ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let mut direction = 1;
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let current = counter_clone.load(Ordering::Relaxed);
                    let next = if current >= 10 {
                        direction = -1;
                        9
                    } else if current <= 0 {
                        direction = 1;
                        1
                    } else {
                        current + direction
                    };
                    counter_clone.store(next, Ordering::Relaxed);
                    egui_ctx.request_repaint();
                }
            });

            Box::new(DemoApp {
                counter,
                name: "Aurora".to_string(),
                current_page: Page::Counter,
                show_menu: false,
                transparent_bg: false,
                original_panel_fill,
                env_filter: String::new(),

                notification_service: None,
                notif_title: "Hello from egui".to_string(),
                notif_body: "This is a test notification".to_string(),
                notif_urgency: 1,
                last_notification_id: None,
                notif_status: String::new(),
                notif_capabilities: String::new(),
                notif_server_info: String::new(),

                runtime_dir_root: aurora_services::package_info::runtime_dir()
                    .unwrap_or_else(|| "/tmp".to_string()),
                runtime_dir_current: aurora_services::package_info::runtime_dir()
                    .unwrap_or_else(|| "/tmp".to_string()),
                runtime_dir_entries: Vec::new(),
                runtime_dir_filter: String::new(),
            })
        }),
    )
    .unwrap();
}
