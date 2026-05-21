use aurora_app::window::MainWindow;
use glutin::context::PossiblyCurrentGlContext;

use crate::gl_context::{AppGlContext, WindowSurface};
use std::sync::Arc;

/// Manages the cover window (app switcher view) for Aurora OS.
///
/// The cover window is a separate winit window with `CATEGORY=cover` that is
/// shown by the compositor when the app is backgrounded.
///
/// Shares the application's GL context with the main window — only the
/// `winit::Window` and `glutin::Surface` are unique to the cover.
pub struct CoverWindow {
    aurora_window: aurora_app::window::CoverWindow,
    surface: WindowSurface,
    egui_glow: egui_glow::EguiGlow,
}

impl CoverWindow {
    /// Create a new cover window and initialize its egui context.
    ///
    /// # Safety
    /// Must be called from the main thread with a valid event loop.
    pub unsafe fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        parent: &MainWindow,
        app_gl: &AppGlContext,
        pixel_ratio: f32,
    ) -> crate::error::Result<Self> {
        let (cover_window_raw, surface) =
            unsafe { crate::gl_context::create_cover_surface(event_loop, app_gl) }?;

        let egui_glow =
            egui_glow::EguiGlow::new(event_loop, Arc::clone(&app_gl.glow), None, None, true);
        egui_glow.egui_ctx.set_pixels_per_point(pixel_ratio);
        egui_glow
            .egui_ctx
            .global_style_mut(|sm| sm.visuals.panel_fill = egui::Color32::TRANSPARENT);

        // Wrap in aurora_app::CoverWindow and link to main window
        let aurora_window = aurora_app::window::CoverWindow::new(cover_window_raw);
        aurora_window.link_to_main(parent);

        // Make the shared GL context current on the cover surface so
        // egui can initialize its painter resources here.
        app_gl
            .gl_context
            .make_current(&surface.gl_surface)
            .expect("failed to make cover context current");

        Ok(Self {
            aurora_window,
            surface,
            egui_glow,
        })
    }

    /// Render the cover UI by calling the app's [`cover_ui`](crate::App::cover_ui) callback.
    pub fn render(&mut self, app_gl: &AppGlContext, app: &mut dyn crate::App) {
        // Make the shared GL context current on the cover surface before rendering
        app_gl
            .gl_context
            .make_current(&self.surface.gl_surface)
            .expect("failed to make cover context current");

        let window = self.aurora_window.window();

        self.egui_glow.run(window, |ui| {
            app.cover_ui(ui);
        });

        unsafe {
            use glow::HasContext;
            let clear_color = egui::Color32::TRANSPARENT.to_normalized_gamma_f32();
            app_gl.glow.clear_color(
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            );
            app_gl.glow.clear(glow::COLOR_BUFFER_BIT);
        }

        self.egui_glow.paint(window);
        self.surface.swap_buffers(&app_gl.gl_context).unwrap();
    }

    /// Handle a winit window event for the cover window.
    pub fn on_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
    ) -> egui_glow::EventResponse {
        self.egui_glow
            .on_window_event(self.aurora_window.window(), event)
    }

    /// Resize the cover surface.
    pub fn resize(&self, app_gl: &AppGlContext, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(&app_gl.gl_context, physical_size);
    }

    /// Request a redraw of the cover window.
    pub fn request_redraw(&self) {
        self.aurora_window.window().focus_window();
        self.aurora_window.window().request_redraw();
    }

    /// Access the winit window.
    pub fn window(&self) -> &winit::window::Window {
        self.aurora_window.window()
    }

    /// Access the underlying Aurora cover window.
    pub fn aurora_window(&self) -> &aurora_app::window::CoverWindow {
        &self.aurora_window
    }

    /// Get the window ID.
    pub fn window_id(&self) -> winit::window::WindowId {
        self.aurora_window.window().id()
    }

    /// Access the egui context.
    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_glow.egui_ctx
    }

    /// Refresh Aurora compositor properties on this cover window.
    pub fn refresh_properties(&self) {
        self.aurora_window.refresh_properties();
    }

    /// Destroy the egui context.
    pub fn destroy(mut self) {
        self.egui_glow.destroy();
    }
}
