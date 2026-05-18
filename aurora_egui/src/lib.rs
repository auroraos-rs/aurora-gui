/*!
# Aurora egui

A library for building [egui](https://github.com/emilk/egui) applications on [Aurora OS](https://auroraos.ru/).

Works on both Aurora OS devices and desktop Linux — the same `App` implementation
runs everywhere without changes.

## Quick Start

```rust,no_run
use aurora_egui::{App, CoverAction, Frame, NativeOptions};

struct MyApp {
    counter: i32,
}

impl App for MyApp {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("My Aurora App");
            ui.label(format!("Count: {}", self.counter));
            if ui.button("Increment").clicked() {
                self.counter += 1;
            }
        });
    }

    fn cover_ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(format!("Counter: {}", self.counter));
        });
    }

    fn cover_actions(&self) -> Vec<CoverAction> {
        vec![CoverAction {
            id: "reset".to_string(),
            icon: "icon-m-refresh".to_string(),
        }]
    }
}

fn main() {
    let options = NativeOptions::default();
    aurora_egui::run_native(
        "MyApp",
        options,
        Box::new(|_cc| Box::new(MyApp { counter: 0 })),
    ).unwrap();
}
```
*/

pub mod app;
pub mod config;
pub mod cover;
pub mod error;
pub mod fonts;
pub mod frame;
pub mod gl_context;
pub mod runner;

pub use app::{App, Storage};
pub use aurora_app::types::CoverAction;
pub use config::{CreationContext, NativeOptions};
pub use error::{AppError, Result};
pub use frame::Frame;

use runner::{AuroraRunner, UserEvent};

/// Run the native application.
///
/// Creates the main window, event loop, and optional cover window, then runs the app.
/// Works on both Aurora OS and desktop Linux.
pub fn run_native(
    _app_name: &str,
    native_options: NativeOptions,
    app_creator: Box<dyn FnOnce(&CreationContext) -> Box<dyn App>>,
) -> Result<()> {
    let event_loop = winit::event_loop::EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(|e| AppError::EventLoop(e.to_string()))?;
    let proxy = event_loop.create_proxy();

    let mut runner = AuroraRunner::new(proxy, app_creator, native_options);
    event_loop
        .run_app(&mut runner)
        .map_err(|e| AppError::EventLoop(e.to_string()))?;

    Ok(())
}
