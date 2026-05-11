use crate::config::AppConfig;
use crate::gl_context::{create_display, GlutinWindowContext};
use crate::ui::{draw_ui, UiState};
use egui_glow;
use std::sync::Arc;

#[derive(Debug)]
pub enum UserEvent {
    Redraw(std::time::Duration),
}

pub struct GlowApp {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    gl_window: Option<GlutinWindowContext>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    repaint_delay: std::time::Duration,
    ui_state: UiState,
}

impl GlowApp {
    pub fn new(proxy: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self {
            proxy,
            gl_window: None,
            gl: None,
            egui_glow: None,
            repaint_delay: std::time::Duration::MAX,
            ui_state: UiState::default(),
        }
    }

    fn redraw(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let mut quit = false;

        self.egui_glow
            .as_mut()
            .unwrap()
            .run(self.gl_window.as_ref().unwrap().window(), |ui| {
                quit = draw_ui(ui, &mut self.ui_state);
            });

        if quit {
            event_loop.exit();
            return;
        }

        event_loop.set_control_flow(if self.repaint_delay.is_zero() {
            self.gl_window.as_ref().unwrap().window().request_redraw();
            winit::event_loop::ControlFlow::Poll
        } else if let Some(repaint_after_instant) =
            std::time::Instant::now().checked_add(self.repaint_delay)
        {
            winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
        } else {
            winit::event_loop::ControlFlow::Wait
        });

        {
            unsafe {
                use glow::HasContext;
                self.gl.as_ref().unwrap().clear_color(
                    self.ui_state.clear_color[0],
                    self.ui_state.clear_color[1],
                    self.ui_state.clear_color[2],
                    1.0,
                );
                self.gl.as_ref().unwrap().clear(glow::COLOR_BUFFER_BIT);
            }

            self.egui_glow
                .as_mut()
                .unwrap()
                .paint(self.gl_window.as_ref().unwrap().window());

            self.gl_window.as_ref().unwrap().swap_buffers().unwrap();
            self.gl_window.as_ref().unwrap().window().set_visible(true);
        }
    }
}

impl winit::application::ApplicationHandler<UserEvent> for GlowApp {
    fn resumed(&mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let (gl_window, gl) = unsafe { create_display(event_loop) }
            .expect("failed to create display");
        let gl = Arc::new(gl);
        gl_window.window().set_visible(true);

        let egui_glow = egui_glow::EguiGlow::new(
            event_loop,
            Arc::clone(&gl),
            None,
            None,
            true,
        );
        egui_glow.egui_ctx.set_pixels_per_point(AppConfig::PIXELS_PER_POINT);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::Redraw(info.delay))
                    .expect("Cannot send event");
            });

        self.gl_window = Some(gl_window);
        self.gl = Some(gl);
        self.egui_glow = Some(egui_glow);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;

        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            self.redraw(event_loop);
            return;
        }

        if let WindowEvent::Resized(physical_size) = &event {
            self.gl_window.as_ref().unwrap().resize(*physical_size);
        }

        let event_response = self
            .egui_glow
            .as_mut()
            .unwrap()
            .on_window_event(self.gl_window.as_ref().unwrap().window(), &event);

        if event_response.repaint {
            self.gl_window.as_ref().unwrap().window().request_redraw();
        }
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: UserEvent,
    ) {
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
            self.gl_window.as_ref().unwrap().window().request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(mut egui_glow) = self.egui_glow.take() {
            egui_glow.destroy();
        }
    }
}
