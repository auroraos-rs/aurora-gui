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
}

struct DemoApp {
    counter: Arc<AtomicI32>,
    name: String,
    current_page: Page,
    show_menu: bool,
    transparent_bg: bool,
    original_panel_fill: egui::Color32,
    env_filter: String,
}

impl App for DemoApp {
    fn update(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
        egui::Panel::top("top_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰").clicked() {
                    self.show_menu = !self.show_menu;
                }
                ui.heading(match self.current_page {
                    Page::Counter => "Counter",
                    Page::EnvVars => "Env Variables",
                    Page::Settings => "Settings",
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
                    });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| match self.current_page {
            Page::Counter => self.show_counter_page(ui),
            Page::EnvVars => self.show_env_vars_page(ui),
            Page::Settings => self.show_settings_page(ui, frame),
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
            })
        }),
    )
    .unwrap();
}
