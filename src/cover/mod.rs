use crate::gl_context::CoverGlContext;
use crate::platform::q_variant;
use std::sync::Arc;

/// Manages the cover window (app switcher view) for Aurora OS.
///
/// The cover window is a separate winit window with `CATEGORY=cover` that is
/// shown by the compositor when the app is backgrounded.
pub struct CoverWindow {
    gl_window: CoverGlContext,
    egui_glow: egui_glow::EguiGlow,
    window_id: winit::window::WindowId,
}

impl CoverWindow {
    /// Create a new cover window and initialize its egui context.
    ///
    /// # Safety
    /// Must be called from the main thread with a valid event loop.
    pub unsafe fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> crate::error::Result<Self> {
        let (gl_window, gl) = unsafe { crate::gl_context::create_cover_display(event_loop) }?;
        let gl = Arc::new(gl);

        // Set cover window properties for the Aurora compositor
        let win_id: u64 = gl_window.window.id().into();
        gl_window
            .window
            .update_generic_property("CATEGORY", q_variant::from_str("cover"));
        gl_window
            .window
            .update_generic_property("WINID", q_variant::from_u64(win_id));
        gl_window
            .window
            .update_generic_property("TRANSPARENT", q_variant::from_bool(false));

        let egui_glow = egui_glow::EguiGlow::new(event_loop, Arc::clone(&gl), None, None, true);
        egui_glow.egui_ctx.set_pixels_per_point(2.0);

        let window_id = gl_window.window.id();

        Ok(Self {
            gl_window,
            egui_glow,
            window_id,
        })
    }

    /// Link this cover window to a main window.
    ///
    /// Sets the `SAILFISH_HAVE_COVER` and `SAILFISH_COVER_WINDOW` properties
    /// on the main window so the compositor knows about the cover.
    pub fn link_to_main(&self, main_window: &mut aurora_app::window::MainWindow) {
        let cover_win_id: u64 = self.gl_window.window.id().into();
        main_window
            .window()
            .update_generic_property("SAILFISH_HAVE_COVER", q_variant::from_bool(true));
        main_window.window().update_generic_property(
            "SAILFISH_COVER_WINDOW",
            q_variant::from_str(&format!("__winref:{}", cover_win_id)),
        );
    }

    /// Render the cover UI by calling the app's [`cover_ui`](crate::App::cover_ui) callback.
    pub fn render(&mut self, app: &mut dyn crate::App, gl: &Arc<glow::Context>) {
        use glutin::context::PossiblyCurrentGlContext;

        // Make cover context current before rendering
        self.gl_window
            .gl_context
            .make_current(&self.gl_window.gl_surface)
            .expect("failed to make cover context current");

        let window = &self.gl_window.window;

        self.egui_glow.run(window, |ctx| {
            app.cover_ui(ctx);
        });

        unsafe {
            use glow::HasContext;
            gl.clear_color(0., 0., 0., 0.);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.egui_glow.paint(window);
        self.gl_window.swap_buffers().unwrap();
    }

    /// Handle a winit window event for the cover window.
    pub fn on_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
    ) -> egui_glow::EventResponse {
        self.egui_glow
            .on_window_event(&self.gl_window.window, event)
    }

    /// Resize the cover surface.
    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.gl_window.resize(physical_size);
    }

    /// Request a redraw of the cover window.
    pub fn request_redraw(&self) {
        self.gl_window.window.request_redraw();
    }

    /// Access the winit window.
    pub fn window(&self) -> &winit::window::Window {
        &self.gl_window.window
    }

    /// Get the window ID.
    pub fn window_id(&self) -> winit::window::WindowId {
        self.window_id
    }

    /// Destroy the egui context.
    pub fn destroy(mut self) {
        self.egui_glow.destroy();
    }
}
