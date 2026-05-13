/// Trait for Aurora egui applications.
///
/// All methods have default implementations, so you only need to implement what you need.
/// The minimum requirement is [`App::update`](Self::update).
///
/// # Example
///
/// ```rust,ignore
/// use aurora_egui::{App, Frame};
///
/// struct MyApp;
///
/// impl App for MyApp {
///     fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
///         egui::CentralPanel::default().show(ctx, |ui| {
///             ui.label("Hello, Aurora!");
///         });
///     }
/// }
/// ```
pub trait App {
    /// Called each frame to draw the main UI (**required**).
    ///
    /// Put your widgets into [`egui::Panel`], [`egui::CentralPanel`], [`egui::Window`], etc.
    fn update(&mut self, ctx: &egui::Context, frame: &mut crate::Frame);

    // -- Lifecycle (optional) --

    /// Called before shutdown to save state.
    ///
    /// Only called if persistent storage is available.
    fn save(&mut self, _storage: &mut dyn crate::Storage) {}

    /// Called once on shutdown, after [`save`](Self::save).
    fn on_exit(&mut self) {}

    /// Background clear color.
    ///
    /// Defaults to dark gray.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    // -- Aurora-specific (optional) --

    /// Render the cover page (app switcher view).
    ///
    /// Called when the app is backgrounded and the cover window is shown.
    /// Use this to display a summary or quick actions.
    fn cover_ui(&mut self, _ctx: &egui::Context) {}

    /// Define cover action buttons.
    ///
    /// These buttons are rendered by the Aurora OS compositor on the cover page.
    /// Click callbacks are not yet supported in v1 (display only).
    fn cover_actions(&self) -> Vec<crate::CoverAction> {
        vec![]
    }
}

/// Persistent storage trait.
///
/// Implemented by the integration to provide load/save of app state.
/// On Aurora OS this may be backed by the file system.
pub trait Storage {
    /// Get the value for the given key.
    fn get_string(&self, key: &str) -> Option<String>;

    /// Set the value for the given key.
    fn set_string(&mut self, key: &str, value: String);

    /// Flush changes to disk.
    fn flush(&mut self);
}
