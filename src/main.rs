use aurora_egui::{App, CoverAction, Frame, NativeOptions};
use egui::RichText;

struct DemoApp {
    counter: i32,
    name: String,
    show_settings: bool,
}

impl App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::Panel::top("menu")
            .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Aurora egui Demo");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Settings").clicked() {
                            self.show_settings = !self.show_settings;
                        }
                    });
                });
            });

        if self.show_settings {
            egui::Panel::right("settings")
                .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    if ui.button("Toggle Status Bar").clicked() {
                        frame.set_statusbar_visible(!frame.is_statusbar_visible());
                    }
                    if ui.button("Toggle Background").clicked() {
                        frame.set_background_visible(!frame.is_background_visible());
                    }
                    if ui.button("Reset Counter").clicked() {
                        self.counter = 0;
                    }
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(format!("Hello, {}!", self.name));
                    ui.add_space(20.0);

                    ui.heading(format!("Counter: {}", self.counter));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("-").clicked() {
                            self.counter -= 1;
                        }
                        if ui.button("+").clicked() {
                            self.counter += 1;
                        }
                    });

                    ui.add_space(20.0);
                    ui.label("The cover page will show this counter when the app is backgrounded.");
                });
            });
    }

    fn cover_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.heading(RichText::new("Aurora egui Demo").size(40.));
                    ui.add_space(20.0);
                    ui.label(RichText::new(format!("Counter: {}", self.counter)).size(32.));
                });
            });
    }

    fn cover_actions(&self) -> Vec<CoverAction> {
        vec![
            CoverAction {
                id: "reset".to_string(),
                icon: "image://theme/icon-cover-previous".to_string(),
                label: "Reset".to_string(),
            },
            CoverAction {
                id: "add".to_string(),
                icon: "image://theme/icon-cover-next".to_string(),
                label: "Add".to_string(),
            },
        ]
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 800.0])
            .with_app_id("com.lmaxyz.egui_aurora_app"),
        ..Default::default()
    };

    aurora_egui::run_native(
        "Aurora egui Demo",
        options,
        Box::new(|_cc| {
            Box::new(DemoApp {
                counter: 0,
                name: "Aurora".to_string(),
                show_settings: false,
            })
        }),
    )
    .unwrap();
}
