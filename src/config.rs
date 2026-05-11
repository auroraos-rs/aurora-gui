/// Application configuration constants.
pub struct AppConfig;

impl AppConfig {
    /// Default window width in logical pixels.
    pub const WINDOW_WIDTH: f32 = 800.0;

    /// Default window height in logical pixels.
    pub const WINDOW_HEIGHT: f32 = 600.0;

    /// Window title shown before first paint.
    pub const WINDOW_TITLE: &str = "egui_glow example";

    /// HiDPI scaling factor for Aurora OS displays.
    pub const PIXELS_PER_POINT: f32 = 1.5;

    /// Default background clear color [r, g, b].
    pub const CLEAR_COLOR: [f32; 3] = [0.1, 0.1, 0.1];

    /// Whether the status bar should be visible.
    pub const STATUSBAR_VISIBLE: bool = true;

    /// Whether the system background should be visible.
    pub const BACKGROUND_VISIBLE: bool = true;
}
