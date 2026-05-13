/// Cover action button definition.
///
/// Action buttons appear on the cover page (app switcher view) when the app is backgrounded.
/// The compositor renders them; they are not drawn by egui.
///
/// # Icon names
///
/// Use Sailfish/Aurora OS icon theme names:
///
/// | Icon name | Meaning |
/// |-----------|---------|
/// | `icon-m-refresh` | Refresh/reload |
/// | `icon-m-add` | Add/plus |
/// | `icon-m-remove` | Remove/delete |
/// | `icon-m-play` | Play |
/// | `icon-m-pause` | Pause |
/// | `icon-m-stop` | Stop |
/// | `icon-m-favorite` | Favorite/star |
/// | `icon-m-share` | Share |
#[derive(Debug, Clone)]
pub struct CoverAction {
    /// Unique identifier for this action.
    ///
    /// Used to construct the D-Bus interface path:
    /// `com.example.app.cover/{id}`.
    pub id: String,

    /// Icon name from the Sailfish/Aurora OS theme.
    pub icon: String,

    /// Human-readable label (not always displayed by the compositor).
    pub label: String,
}

impl CoverAction {
    /// Create a new cover action.
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
        }
    }
}
