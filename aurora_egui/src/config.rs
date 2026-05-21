use std::path::PathBuf;

/// Options for [`run_native`](crate::run_native).
///
/// These control the initial window state, rendering options, and Aurora-specific behavior.
#[derive(Debug, Clone)]
pub struct NativeOptions {
    // -- Same as eframe::NativeOptions --
    /// Controls the native window of the root viewport.
    ///
    /// This is where you set things like window title, size, and app id.
    pub viewport: egui::ViewportBuilder,

    /// Turn on vertical syncing, limiting the FPS to the display refresh rate.
    ///
    /// The default is `true`.
    pub vsync: bool,

    /// On desktop: make the window position centered at initialization.
    pub centered: bool,

    /// Controls whether or not the native window position and size will be persisted.
    pub persist_window: bool,

    /// The folder where app state will be stored.
    pub persistence_path: Option<PathBuf>,

    // -- Aurora-specific --
    /// Show the system status bar initially (default: `true`).
    ///
    /// When `true`, the library reserves space at the top of the window
    /// so your UI is not drawn underneath the status bar.
    pub statusbar_visible: bool,

    /// Enable cover page rendering (default: `true`).
    ///
    /// When `true`, a separate cover window is created and [`App::cover_ui`](crate::App::cover_ui)
    /// is called when the app is backgrounded.
    pub enable_cover_page: bool,

    /// Automatically apply system font settings from DConf (default: `true`).
    ///
    /// Only used on Aurora OS.
    pub use_system_fonts: bool,
}

impl Default for NativeOptions {
    fn default() -> Self {
        Self {
            viewport: egui::ViewportBuilder::default(),
            vsync: true,
            centered: true,
            persist_window: true,
            persistence_path: None,
            statusbar_visible: true,
            enable_cover_page: true,
            use_system_fonts: true,
        }
    }
}

/// Context passed to the app creator closure in [`run_native`](crate::run_native).
///
/// Use this to set up fonts, load state, or initialize OpenGL resources before the app starts.
pub struct CreationContext {
    /// The egui context.
    ///
    /// You can use this to customize the look of egui before the first frame.
    pub egui_ctx: egui::Context,

    /// The OpenGL context, if available.
    pub gl: Option<std::sync::Arc<glow::Context>>,

    /// Persistent storage, if available.
    pub storage: Option<Box<dyn crate::Storage>>,

    /// The system pixel ratio (e.g. `1.5` for HiDPI displays).
    pub pixel_ratio: f32,

    /// The height of the system status bar in logical pixels.
    pub statusbar_height: f32,
}
