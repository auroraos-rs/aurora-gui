use crate::cover::CoverWindow;
use crate::gl_context::{GlutinWindowContext, create_display};
use crate::platform::q_variant;
use crate::{App, CreationContext, Frame, NativeOptions};
use std::sync::Arc;

#[derive(Debug)]
pub enum UserEvent {
    Redraw(std::time::Duration),
}

/// Main event loop runner for Aurora OS.
pub struct AuroraRunner {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    app_creator: Option<Box<dyn FnOnce(&CreationContext) -> Box<dyn App>>>,
    app: Option<Box<dyn App>>,

    // Main window
    main_gl_window: Option<GlutinWindowContext>,
    main_gl: Option<Arc<glow::Context>>,
    main_egui_glow: Option<egui_glow::EguiGlow>,
    main_window_id: Option<winit::window::WindowId>,

    // Cover window
    cover_window: Option<CoverWindow>,

    repaint_delay: std::time::Duration,
    frame: Frame,
    native_options: NativeOptions,
    statusbar_visible: bool,
    background_visible: bool,
    focused: bool,
    pixel_ratio: f32,
    statusbar_height: f32,
}

impl AuroraRunner {
    pub fn new(
        proxy: winit::event_loop::EventLoopProxy<UserEvent>,
        app_creator: Box<dyn FnOnce(&CreationContext) -> Box<dyn App>>,
        native_options: NativeOptions,
    ) -> Self {
        Self {
            proxy,
            app_creator: Some(app_creator),
            app: None,
            main_gl_window: None,
            main_gl: None,
            main_egui_glow: None,
            main_window_id: None,
            cover_window: None,
            repaint_delay: std::time::Duration::MAX,
            frame: Frame {
                statusbar_visible: native_options.statusbar_visible,
                background_visible: native_options.background_visible,
                gl: None,
            },
            statusbar_visible: native_options.statusbar_visible,
            background_visible: native_options.background_visible,
            native_options,
            focused: true,
            pixel_ratio: 2.0,
            statusbar_height: 41.0,
        }
    }

    fn init_app(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // -- Main window --
        let (mut main_gl_window, main_gl) =
            unsafe { create_display(event_loop) }.expect("failed to create main display");
        let main_gl = Arc::new(main_gl);
        main_gl_window.window().set_visible(true);

        let main_egui_glow =
            egui_glow::EguiGlow::new(event_loop, Arc::clone(&main_gl), None, None, true);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        main_egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::Redraw(info.delay))
                    .expect("Cannot send event");
            });

        if self.native_options.use_pixel_ratio {
            main_egui_glow
                .egui_ctx
                .set_pixels_per_point(self.pixel_ratio);
        }

        let main_window_id = main_gl_window.window().id();

        // -- Cover window --
        let cover_window =
            unsafe { CoverWindow::new(event_loop) }.expect("failed to create cover window");

        // Link cover to main window
        cover_window.link_to_main(main_gl_window.main_window_mut());

        // Register cover actions on the main window
        let app = self.app.as_mut().unwrap();
        let actions = app.cover_actions();
        let app_id = self
            .native_options
            .viewport
            .app_id
            .as_deref()
            .unwrap_or("com.example.app");
        let mut action_str = String::new();
        for (i, action) in actions.iter().enumerate() {
            let dbus_interface =
                format!("org.sailfishos.coveraction.{}.pid1.id{}", app_id, i + 1);
            action_str.push_str(&format!(
                "{}\ntrigger\n{}\n{}\n",
                dbus_interface, i, action.icon
            ));
        }
        action_str.push('\0');
        main_gl_window
            .main_window_mut()
            .window()
            .update_generic_property(
                "_APP_COVER_ACTION",
                q_variant::from_str(&action_str),
            );

        self.frame.gl = Some(Arc::clone(&main_gl));
        self.main_gl_window = Some(main_gl_window);
        self.main_gl = Some(main_gl);
        self.main_egui_glow = Some(main_egui_glow);
        self.main_window_id = Some(main_window_id);
        self.cover_window = Some(cover_window);
    }

    fn redraw_main(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        use glutin::context::PossiblyCurrentGlContext;

        let app = self.app.as_mut().unwrap();
        let main_egui_glow = self.main_egui_glow.as_mut().unwrap();
        let main_gl_window = self.main_gl_window.as_mut().unwrap();

        // Make main context current
        main_gl_window
            .gl_context
            .make_current(&main_gl_window.gl_surface)
            .expect("failed to make main context current");

        // Update window settings if frame flags changed
        if self.frame.statusbar_visible != self.statusbar_visible {
            self.statusbar_visible = self.frame.statusbar_visible;
            main_gl_window
                .main_window_mut()
                .set_statusbar_visible(self.statusbar_visible);
        }
        if self.frame.background_visible != self.background_visible {
            self.background_visible = self.frame.background_visible;
            main_gl_window
                .main_window_mut()
                .set_background_visible(self.background_visible);
        }

        let window = main_gl_window.window();

        main_egui_glow.run(window, |ctx| {
            if self.statusbar_visible {
                egui::TopBottomPanel::top("aurora_statusbar")
                    .exact_height(self.statusbar_height)
                    .show_separator_line(false)
                    .show(ctx, |_ui| {});
            }

            app.update(ctx, &mut self.frame);
        });

        event_loop.set_control_flow(if self.repaint_delay.is_zero() {
            window.request_redraw();
            winit::event_loop::ControlFlow::Poll
        } else if let Some(repaint_after_instant) =
            std::time::Instant::now().checked_add(self.repaint_delay)
        {
            winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
        } else {
            winit::event_loop::ControlFlow::Wait
        });

        unsafe {
            use glow::HasContext;
            let clear_color = app.clear_color(&main_egui_glow.egui_ctx.global_style().visuals);
            self.main_gl.as_ref().unwrap().clear_color(
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            );
            self.main_gl.as_ref().unwrap().clear(glow::COLOR_BUFFER_BIT);
        }

        main_egui_glow.paint(window);
        main_gl_window.swap_buffers().unwrap();
        window.set_visible(true);
    }

    fn redraw_cover(&mut self) {
        let app = self.app.as_mut().unwrap();
        let cover = self.cover_window.as_mut().unwrap();
        cover.render(app.as_mut(), self.main_gl.as_ref().unwrap());
    }

    fn redraw(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
    ) {
        if self.main_window_id == Some(window_id) {
            self.redraw_main(event_loop);
        } else if self.cover_window.as_ref().map(|c| c.window_id()) == Some(window_id) {
            self.redraw_cover();
        }
    }
}

impl winit::application::ApplicationHandler<UserEvent> for AuroraRunner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Create the app first so we can get cover actions
        let creation_context = CreationContext {
            egui_ctx: egui::Context::default(),
            gl: None,
            storage: None,
            pixel_ratio: self.pixel_ratio,
            statusbar_height: self.statusbar_height,
            font_settings: None,
        };
        let app_creator = self
            .app_creator
            .take()
            .expect("app_creator already consumed");
        let app = app_creator(&creation_context);
        self.app = Some(app);

        self.init_app(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;

        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            self.redraw(event_loop, window_id);
            return;
        }

        if let WindowEvent::Focused(focused) = &event {
            if self.main_window_id == Some(window_id) {
                self.focused = *focused;
                if !focused {
                    // Request redraw of cover window when main loses focus
                    if let Some(cover) = &self.cover_window {
                        cover.request_redraw();
                    }
                }
            }
        }

        if self.main_window_id == Some(window_id) {
            if let WindowEvent::Resized(physical_size) = &event {
                self.main_gl_window.as_ref().unwrap().resize(*physical_size);
            }

            let event_response = self
                .main_egui_glow
                .as_mut()
                .unwrap()
                .on_window_event(self.main_gl_window.as_ref().unwrap().window(), &event);

            if event_response.repaint {
                self.main_gl_window
                    .as_ref()
                    .unwrap()
                    .window()
                    .request_redraw();
            }
        } else if self.cover_window.as_ref().map(|c| c.window_id()) == Some(window_id) {
            if let WindowEvent::Resized(physical_size) = &event {
                self.cover_window.as_ref().unwrap().resize(*physical_size);
            }

            let event_response = self.cover_window.as_mut().unwrap().on_window_event(&event);

            if event_response.repaint {
                self.cover_window.as_ref().unwrap().request_redraw();
            }
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Redraw(delay) => self.repaint_delay = delay,
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = &cause {
            if let Some(main) = &self.main_gl_window {
                main.window().request_redraw();
            }
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(mut app) = self.app.take() {
            app.on_exit();
        }
        if let Some(mut main_egui_glow) = self.main_egui_glow.take() {
            main_egui_glow.destroy();
        }
        if let Some(cover_window) = self.cover_window.take() {
            cover_window.destroy();
        }
    }
}
