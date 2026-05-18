# aurora_egui

egui integration for Aurora OS.

Part of the [`aurora_app` workspace](../README.md). This crate provides the glue between [egui](https://github.com/emilk/egui) and Aurora OS windowing/services.

Works on both Aurora OS devices and desktop Linux — the same `App` implementation runs everywhere without changes. Handles window creation, GL context setup via `glutin`, system font loading, status bar padding, cover page rendering, and Aurora-specific window properties automatically.

## Usage

```rust
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

## The `App` Trait

| Method | Required | Description |
|--------|----------|-------------|
| `update` | **Yes** | Called every frame to draw the main UI |
| `save` | No | Called before shutdown to persist state |
| `on_exit` | No | Called once on shutdown |
| `cover_ui` | No | Rendered on the cover page when the app is backgrounded |
| `cover_actions` | No | Cover action buttons shown by the compositor |

## `NativeOptions`

```rust
pub struct NativeOptions {
    pub viewport: egui::ViewportBuilder,     // Window title, size, app_id
    pub vsync: bool,                         // Default: true
    pub centered: bool,                      // Center on desktop
    pub persist_window: bool,                // Save/restore window position
    pub persistence_path: Option<PathBuf>,
    pub statusbar_visible: bool,             // Default: true
    pub enable_cover_page: bool,             // Default: true
    pub use_system_fonts: bool,              // Default: true
}
```

## `CreationContext`

Passed to the app creator closure before the first frame:

```rust
pub struct CreationContext {
    pub egui_ctx: egui::Context,
    pub gl: Option<Arc<glow::Context>>,
    pub storage: Option<Box<dyn Storage>>,
    pub pixel_ratio: f32,
    pub statusbar_height: f32,
    pub font_settings: Option<aurora_services::FontSettings>,
}
```

Use `cc.egui_ctx` to customize global styles or load fonts before the app starts.

## `Frame`

Passed to `App::update` each frame:

| Method | Description |
|--------|-------------|
| `set_statusbar_visible(bool)` | Show/hide the system status bar |
| `is_statusbar_visible() -> bool` | Current status bar state |

## System Fonts

When `NativeOptions::use_system_fonts` is `true` (the default), `aurora_egui` automatically:

1. Queries DConf for the system's font family and size preferences
2. Loads the actual font files from the system font database via `fontdb`
3. Registers them in egui as the highest-priority fonts (built-in fonts remain as fallbacks)
4. Sets `TextStyle` sizes to match the Aurora OS settings

This happens before the app creator closure runs, so your app sees the correct fonts immediately.

## Modules

| Module | Purpose |
|--------|---------|
| `app` | `App` and `Storage` traits |
| `config` | `NativeOptions` and `CreationContext` |
| `cover` | `CoverWindow` — separate GL window for the Aurora cover page |
| `error` | `AppError` and `Result` types |
| `fonts` | System font loading via `fontdb` |
| `frame` | Per-frame `Frame` handle |
| `gl_context` | `glutin`/`glow` context creation |
| `runner` | `AuroraRunner` — the winit event loop implementation |

## Cross-Compilation

```bash
# Aurora OS ARM64
cross build --release -p demo --target aarch64-unknown-linux-gnu

# Aurora OS ARMv7
cross build --release -p demo --target armv7-unknown-linux-gnueabihf
```

Requires [`cross`](https://github.com/cross-rs/cross) and the `Cross.toml` in the workspace root.

## License

Apache 2.0
