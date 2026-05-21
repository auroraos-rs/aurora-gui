use super::{AuroraRunner, UserEvent};
use crate::cover::CoverWindow;
use crate::gl_context::create_main_window;
use crate::CreationContext;
use std::sync::Arc;

impl AuroraRunner {
    pub fn init_app(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // -- Main window --
        let mut window_attributes = winit::window::WindowAttributes::default().with_visible(false);

        if let Some(size) = self.platform.native_options.viewport.inner_size {
            window_attributes = window_attributes.with_inner_size(winit::dpi::LogicalSize {
                width: size.x as f64,
                height: size.y as f64,
            });
        }
        if let Some(ref title) = self.platform.native_options.viewport.title {
            window_attributes = window_attributes.with_title(title.clone());
        }
        if let Some(resizable) = self.platform.native_options.viewport.resizable {
            window_attributes = window_attributes.with_resizable(resizable);
        }

        let (main_window_raw, main_surface, app_gl) =
            unsafe { create_main_window(event_loop, window_attributes) }
                .expect("failed to create main window");

        let mut main_window = aurora_app::window::MainWindow::new(main_window_raw);

        // Register IME event handler: background thread → EventLoopProxy → user_event
        let proxy = self.proxy.clone();
        if let Err(e) = main_window.add_ime_event_handler(move |event| {
            let _ = proxy.send_event(super::UserEvent::ImeEvent(event));
        }) {
            log::warn!("Failed to register IME event handler: {}", e);
        }

        main_window.window().set_visible(true);

        // Apply initial frame visibility settings
        main_window.set_statusbar_visible(self.platform.frame.statusbar_visible);
        self.platform.last_applied_statusbar = self.platform.frame.statusbar_visible;

        let main_egui_glow =
            egui_glow::EguiGlow::new(event_loop, Arc::clone(&app_gl.glow), None, None, true);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        main_egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::Redraw(info.delay))
                    .expect("Cannot send event");
            });

        main_egui_glow
            .egui_ctx
            .set_pixels_per_point(self.platform.system_pixel_ratio);

        // Apply system font settings before the app creator runs
        if let Some(ref font_settings) = self.platform.font_settings {
            crate::fonts::apply_font_settings(&main_egui_glow.egui_ctx, font_settings);
        }

        let main_window_id = main_window.window().id();

        self.main_state.rotation = None;
        self.main_state.shapes = Default::default();
        self.main_state.pixels_per_point = self.platform.system_pixel_ratio;
        self.main_state.textures_delta = Default::default();
        self.main_state.viewport_info = Default::default();
        self.main_state.keyboard_area = None;
        self.main_state.pending_ime_events = Vec::new();

        // -- Create app with the REAL egui context --
        let creation_context = CreationContext {
            egui_ctx: main_egui_glow.egui_ctx.clone(),
            gl: Some(Arc::clone(&app_gl.glow)),
            storage: None,
            pixel_ratio: self.platform.system_pixel_ratio,
            statusbar_height: self.platform.statusbar_height,
        };
        let app_creator = self
            .app_state
            .creator
            .take()
            .expect("app_creator already consumed");
        let app = app_creator(&creation_context);

        // -- Cover window --
        let cover_window = unsafe {
            CoverWindow::new(event_loop, &main_window, &app_gl, self.platform.system_pixel_ratio)
        }
        .expect("failed to create cover window");
        cover_window.window().set_visible(true);

        // Refresh cover and main window properties after the cover is mapped,
        // matching the Flutter embedder pattern where UpdateProperties is called
        // after both windows are visible.
        cover_window.refresh_properties();
        main_window.refresh_cover_linkage(cover_window.aurora_window());

        // Apply the same system font settings to the cover context
        if let Some(ref font_settings) = self.platform.font_settings {
            crate::fonts::apply_font_settings(cover_window.egui_ctx(), font_settings);
        }

        // Register cover actions on the main window
        let app_id = self
            .platform
            .native_options
            .viewport
            .app_id
            .as_deref()
            .unwrap_or("com.example.app");
        main_window.set_cover_actions(app_id, &app.cover_actions());

        self.main_state.surface = Some(main_surface);
        self.main_state.aurora_window = Some(main_window);
        self.app_gl = Some(app_gl);
        self.main_state.egui_glow = Some(main_egui_glow);
        self.main_state.window_id = Some(main_window_id);
        self.cover = Some(cover_window);
        self.app_state.app = Some(app);
    }

    pub fn cleanup(&mut self) {
        if let Some(mut app) = self.app_state.app.take() {
            app.on_exit();
        }
        if let Some(mut main_egui_glow) = self.main_state.egui_glow.take() {
            main_egui_glow.destroy();
        }
        if let Some(cover) = self.cover.take() {
            cover.destroy();
        }
        // Clear cover linkage from the main window before destroying the cover,
        // matching the Flutter embedder behavior.
        if let Some(main_window) = &self.main_state.aurora_window {
            main_window.clear_cover_linkage();
        }
        if let Some(ref mut main) = self.main_state.aurora_window {
            main.clear_ime_event_handlers();
            main.hide_ime().ok();
        }
        // GL context and surfaces are dropped when app_gl is dropped.
        self.app_gl = None;
        self.main_state.surface = None;
    }
}
