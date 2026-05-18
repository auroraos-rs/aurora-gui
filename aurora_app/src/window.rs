use winit::window::Window;

use crate::q_variant_compat as qv;
use crate::types::CoverAction;

pub use maliit::MaliitError as ImeError;

// ------------------------------------------------------------------------------
// Free functions — internal helpers, not part of the public API
// ------------------------------------------------------------------------------

fn set_main_window_id(window: &Window) {
    let win_id: u64 = window.id().into();
    window.update_generic_property("WINID", qv::from_u64(win_id));
}

fn set_statusbar_visible(window: &Window, visible: bool) {
    window.update_generic_property("STATUSBAR_VISIBLE", qv::from_bool(visible));
}

fn set_background_visible(window: &Window, visible: bool) {
    window.update_generic_property("BACKGROUND_VISIBLE", qv::from_bool(visible));
}

fn set_cover_window_properties(window: &Window) {
    let win_id: u64 = window.id().into();
    window.update_generic_property("WINID", qv::from_u64(win_id));
    window.update_generic_property("CATEGORY", qv::from_str("cover"));
    window.update_generic_property("TRANSPARENT", qv::from_bool(false));
}

fn set_cover_transparent(window: &Window, transparent: bool) {
    window.update_generic_property("TRANSPARENT", qv::from_bool(transparent));
}

fn link_cover_to_main(main: &Window, cover: &Window) {
    let cover_win_id: u64 = cover.id().into();
    main.update_generic_property("SAILFISH_HAVE_COVER", qv::from_bool(true));
    main.update_generic_property(
        "SAILFISH_COVER_WINDOW",
        qv::from_string(format!("__winref:{}", cover_win_id)),
    );
}

fn clear_cover_linkage(main: &Window) {
    main.update_generic_property("SAILFISH_HAVE_COVER", qv::from_bool(false));
    main.update_generic_property("SAILFISH_COVER_WINDOW", qv::from_str(""));
}

fn refresh_cover_window_properties(window: &Window) {
    set_cover_window_properties(window);
}

fn set_main_window_cover_actions(window: &Window, app_id: &str, actions: &[CoverAction]) {
    let mut action_str = String::new();
    for (i, action) in actions.iter().enumerate() {
        let dbus_interface = format!("org.sailfishos.coveraction.{}.pid1.id{}", app_id, i + 1);
        action_str.push_str(&format!(
            "{}\ntrigger\n{}\n{}\n",
            dbus_interface, i, action.icon
        ));
    }
    action_str.push('\0');
    window.update_generic_property("_APP_COVER_ACTION", qv::from_str(&action_str));
}

// ------------------------------------------------------------------------------
// MainWindow — wraps the primary application window with Aurora OS properties
// ------------------------------------------------------------------------------

/// Wrapper around the main application window that manages Aurora OS properties
/// and IME integration.
pub struct MainWindow {
    window: Window,
    input_method: Option<maliit::input_method::InputMethod>,
}

impl MainWindow {
    pub fn new(window: Window) -> Self {
        set_main_window_id(&window);
        set_background_visible(&window, true);

        let input_method = maliit::input_method::InputMethod::new().ok();
        if input_method.is_none() {
            log::info!("Maliit not available (desktop or no IME server)");
        }

        Self {
            window,
            input_method,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Link a cover window without taking ownership.
    pub fn link_cover(&self, cover: &CoverWindow) {
        link_cover_to_main(&self.window, cover.window());
    }

    /// Refresh cover linkage properties on the main window.
    pub fn refresh_cover_linkage(&self, cover: &Window) {
        link_cover_to_main(&self.window, cover);
    }

    /// Clear cover linkage properties from the main window.
    pub fn clear_cover_linkage(&self) {
        clear_cover_linkage(&self.window);
    }

    pub fn set_statusbar_visible(&self, is_visible: bool) {
        set_statusbar_visible(&self.window, is_visible);
    }

    pub fn set_cover_actions(&self, app_id: &str, actions: &[CoverAction]) {
        set_main_window_cover_actions(&self.window, app_id, actions);
    }

    // -- IME --

    pub fn show_ime(&mut self) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.show()?;
        }
        Ok(())
    }

    /// Show the keyboard after telling Maliit the widget has focus.
    ///
    /// This is the preferred way to open the keyboard — it sends
    /// `focusState=true` so the server knows a text widget is active.
    pub fn show_ime_with_info(&mut self, info: &maliit::input_method::WidgetInfo) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.show_with_info(info)?;
        }
        Ok(())
    }

    /// Update widget information on the Maliit server.
    ///
    /// Call with `focus_changed = true` when the widget gains or loses focus.
    pub fn update_ime_widget_info(
        &mut self,
        info: &maliit::input_method::WidgetInfo,
        focus_changed: bool,
    ) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.update_widget_information(info, focus_changed)?;
        }
        Ok(())
    }

    pub fn hide_ime(&mut self) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.hide()?;
        }
        Ok(())
    }

    pub fn reset_ime(&mut self) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.reset()?;
        }
        Ok(())
    }

    pub fn set_ime_language(&mut self, lang: &str) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.set_language(lang)?;
        }
        Ok(())
    }

    /// Rotate the on-screen keyboard to match the given screen orientation.
    pub fn rotate_ime(&mut self, orientation: maliit::events::Orientation) -> Result<(), ImeError> {
        if let Some(ref mut im) = self.input_method {
            im.rotate(orientation)?;
        }
        Ok(())
    }

    /// Register a callback that is called on a background thread for each IME event.
    ///
    /// The handler is invoked from a dedicated D-Bus thread. To process events on
    /// the main thread, send them via an event-loop proxy (e.g. winit's
    /// `EventLoopProxy`) and queue them for the next frame.
    pub fn add_ime_event_handler<F>(&mut self, handler: F) -> Result<(), ImeError>
    where
        F: Fn(crate::InputMethodEvent) + Send + 'static,
    {
        if let Some(ref mut im) = self.input_method {
            im.add_event_handler(handler)?;
        }
        Ok(())
    }

    /// Remove all registered IME event handlers and stop the background thread.
    pub fn clear_ime_event_handlers(&mut self) {
        if let Some(ref mut im) = self.input_method {
            im.clear_event_handlers();
        }
    }

    /// Consume this wrapper and return the underlying [`Window`].
    pub fn into_inner(self) -> Window {
        self.window
    }
}

// ------------------------------------------------------------------------------
// CoverWindow — wraps the cover window with Aurora OS properties
// ------------------------------------------------------------------------------

/// Wrapper around a cover window that manages Aurora OS properties.
pub struct CoverWindow {
    window: Window,
}

impl CoverWindow {
    pub fn new(window: Window) -> Self {
        set_cover_window_properties(&window);
        Self { window }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Link this cover window to the given main window.
    pub fn link_to_main(&self, main: &MainWindow) {
        link_cover_to_main(main.window(), &self.window);
    }

    pub fn set_transparent(&self, is_transparent: bool) {
        set_cover_transparent(&self.window, is_transparent);
    }

    /// Refresh Aurora compositor properties on this cover window.
    pub fn refresh_properties(&self) {
        refresh_cover_window_properties(&self.window);
    }
}
