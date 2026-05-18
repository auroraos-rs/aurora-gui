/// Platform-agnostic frame handle passed to [`App::update`](crate::App::update) each frame.
///
/// On Aurora OS this provides access to window properties like status bar visibility.
/// On desktop most methods are no-ops for compatibility.
pub struct Frame {
    pub(crate) statusbar_visible: bool,
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

    /// Returns whether the status bar is currently visible.
    pub fn is_statusbar_visible(&self) -> bool {
        self.statusbar_visible
    }
}
