use std::sync::Arc;

/// Platform-agnostic frame handle passed to [`App::update`](crate::App::update) each frame.
///
/// On Aurora OS this provides access to window properties like status bar visibility.
/// On desktop most methods are no-ops for compatibility.
pub struct Frame {
    pub(crate) statusbar_visible: bool,
    pub(crate) background_visible: bool,
    pub(crate) gl: Option<Arc<glow::Context>>,
}

impl Frame {
    /// Show or hide the system status bar.
    ///
    /// When the status bar is visible, the library automatically adds top padding
    /// so your UI is not drawn underneath it.
    ///
    /// # Aurora OS
    /// This updates the `STATUSBAR_VISIBLE` window property.
    ///
    /// # Desktop
    /// No-op.
    pub fn set_statusbar_visible(&mut self, visible: bool) {
        self.statusbar_visible = visible;
    }

    /// Show or hide the system background.
    ///
    /// # Aurora OS
    /// This updates the `BACKGROUND_VISIBLE` window property.
    ///
    /// # Desktop
    /// No-op.
    pub fn set_background_visible(&mut self, visible: bool) {
        self.background_visible = visible;
    }

    /// Returns whether the status bar is currently visible.
    pub fn is_statusbar_visible(&self) -> bool {
        self.statusbar_visible
    }

    /// Returns whether the system background is currently visible.
    pub fn is_background_visible(&self) -> bool {
        self.background_visible
    }

    /// Access the OpenGL context.
    ///
    /// Returns `None` if the app is running on a backend that does not use OpenGL.
    pub fn gl(&self) -> Option<&Arc<glow::Context>> {
        self.gl.as_ref()
    }
}
