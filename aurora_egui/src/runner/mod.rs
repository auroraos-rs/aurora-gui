mod ime;
mod lifecycle;
mod renderer;
mod rotation;

use crate::cover::CoverWindow;
use crate::gl_context::AppGlContext;
use crate::{App, CreationContext, Frame, NativeOptions};
use egui_rotate::Rotation as EguiRotation;

#[derive(Debug)]
pub enum UserEvent {
    Redraw(std::time::Duration),
    ImeEvent(aurora_app::InputMethodEvent),
}

/// State related to the application instance and its creator closure.
struct AppState {
    creator: Option<Box<dyn FnOnce(&CreationContext) -> Box<dyn App>>>,
    app: Option<Box<dyn App>>,
}

/// State related to the main window: GL, egui, rotation, and IME events.
struct MainWindowState {
    surface: Option<crate::gl_context::WindowSurface>,
    aurora_window: Option<aurora_app::window::MainWindow>,
    egui_glow: Option<egui_glow::EguiGlow>,
    window_id: Option<winit::window::WindowId>,
    rotation: EguiRotation,
    shapes: Vec<egui::epaint::ClippedShape>,
    pixels_per_point: f32,
    textures_delta: egui::TexturesDelta,
    viewport_info: egui::ViewportInfo,
    keyboard_area: Option<egui::Rect>,
    pending_ime_events: Vec<aurora_app::InputMethodEvent>,
    last_sent_ime_angle: Option<i32>,
}

/// Platform / environment state that is not tied to a specific window.
struct PlatformState {
    repaint_delay: Option<std::time::Duration>,
    frame: Frame,
    native_options: NativeOptions,
    focused: bool,
    system_pixel_ratio: f32,
    statusbar_height: f32,
    font_settings: Option<aurora_services::FontSettings>,
    last_applied_statusbar: bool,
    ime_enabled: bool,
}

/// Main event loop runner for Aurora OS.
pub struct AuroraRunner {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    app_state: AppState,
    main_state: MainWindowState,
    app_gl: Option<AppGlContext>,
    cover: Option<CoverWindow>,
    platform: PlatformState,
}

impl AuroraRunner {
    pub fn new(
        proxy: winit::event_loop::EventLoopProxy<UserEvent>,
        app_creator: Box<dyn FnOnce(&CreationContext) -> Box<dyn App>>,
        native_options: NativeOptions,
    ) -> Self {
        let (pixel_ratio, statusbar_height, font_settings) =
            aurora_services::SettingsService::new()
                .ok()
                .map(|s| {
                    let pixel_ratio = s.pixel_ratio() as f32;
                    let statusbar_height = s.statusbar_height() as f32;
                    let fonts = if native_options.use_system_fonts {
                        Some(s.font_settings())
                    } else {
                        None
                    };
                    (pixel_ratio, statusbar_height, fonts)
                })
                .unwrap_or_else(|| {
                    (
                        aurora_services::DEFAULT_PIXEL_RATIO as f32,
                        aurora_services::DEFAULT_STATUSBAR_HEIGHT as f32,
                        None,
                    )
                });

        Self {
            proxy,
            app_state: AppState {
                creator: Some(app_creator),
                app: None,
            },
            main_state: MainWindowState {
                surface: None,
                aurora_window: None,
                egui_glow: None,
                window_id: None,
                rotation: EguiRotation::None,
                shapes: Default::default(),
                pixels_per_point: pixel_ratio,
                textures_delta: Default::default(),
                viewport_info: Default::default(),
                keyboard_area: None,
                pending_ime_events: Vec::new(),
                last_sent_ime_angle: None,
            },
            app_gl: None,
            cover: None,
            platform: PlatformState {
                repaint_delay: None,
                frame: Frame {
                    statusbar_visible: native_options.statusbar_visible,
                    rotation: EguiRotation::None,
                },
                native_options: native_options.clone(),
                focused: true,
                system_pixel_ratio: pixel_ratio,
                statusbar_height,
                font_settings,
                last_applied_statusbar: native_options.statusbar_visible,
                ime_enabled: false,
            },
        }
    }

    fn is_main_window(&self, window_id: winit::window::WindowId) -> bool {
        self.main_state.window_id == Some(window_id)
    }

    fn is_cover_window(&self, window_id: winit::window::WindowId) -> bool {
        self.cover.as_ref().map(|c| c.window_id()) == Some(window_id)
    }

    fn handle_main_window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: &winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;

        if let WindowEvent::Transformed(transformed) = event {
            let rotation = rotation::transform_to_rotation(transformed);
            self.main_state.rotation = rotation;
            self.platform.frame.rotation = rotation;
            if let Some(ref mut main) = self.main_state.aurora_window {
                // winit Transform represents the compositor's counter-rotation to keep
                // content upright. Inverting gives the physical device orientation.
                let orientation = match transformed {
                    winit::event::Transform::Normal => aurora_app::Orientation::Portrait,
                    winit::event::Transform::_90 => aurora_app::Orientation::LandscapeFlipped,
                    winit::event::Transform::_180 => aurora_app::Orientation::PortraitFlipped,
                    winit::event::Transform::_270 => aurora_app::Orientation::Landscape,
                    _ => aurora_app::Orientation::Portrait,
                };
                // Deduplicate: the compositor may send Transformed repeatedly
                // during a rotation animation. Only notify Maliit when the
                // orientation actually changes.
                let angle = orientation as i32;
                if self.main_state.last_sent_ime_angle != Some(angle) {
                    self.main_state.last_sent_ime_angle = Some(angle);
                    main.rotate_ime(orientation).ok();
                }
                main.window().request_redraw();
            }
        }

        if let WindowEvent::Focused(focused) = event {
            self.platform.focused = *focused;
            if !focused {
                // Tell Maliit focus is lost, then hide keyboard and drop input focus
                if let Some(ref mut main) = self.main_state.aurora_window {
                    let info = aurora_app::WidgetInfo {
                        focus_state: false,
                        ..Default::default()
                    };
                    main.update_ime_widget_info(&info, true).ok();
                    main.hide_ime().ok();
                }
                self.main_state.keyboard_area = None;
                if let Some(main) = self.main_state.egui_glow.as_ref() {
                    let focused_id = main.egui_ctx.memory(|mem| mem.focused());
                    if let Some(id) = focused_id {
                        main.egui_ctx.memory_mut(|mem| mem.surrender_focus(id));
                    }
                }
                // Request redraw of cover window when main loses focus
                if let Some(cover) = &self.cover {
                    cover.request_redraw();
                }
            }
        }

        match event {
            WindowEvent::Ime(winit::event::Ime::Enabled) => {
                self.platform.ime_enabled = true;
                if let Some(ref mut main) = self.main_state.aurora_window {
                    // ToDo: Add support for different types of keyboard
                    let info = aurora_app::WidgetInfo {
                        focus_state: true,
                        cursor_position: 1,
                        ..Default::default()
                    };
                    main.show_ime_with_info(&info).ok();
                }
            }
            WindowEvent::Ime(winit::event::Ime::Disabled) => {
                self.platform.ime_enabled = false;
                if let Some(ref mut main) = self.main_state.aurora_window {
                    // let info = aurora_app::WidgetInfo {
                    //     focus_state: false,
                    //     ..Default::default()
                    // };
                    // main.update_ime_widget_info(&info, true).ok();
                    main.hide_ime().ok();
                }
                self.main_state.keyboard_area = None;
            }
            _ => {}
        }

        if let WindowEvent::Resized(physical_size) = event {
            self.main_state
                .surface
                .as_ref()
                .unwrap()
                .resize(&self.app_gl.as_ref().unwrap().gl_context, *physical_size);
        }

        let event_response = self.main_state.egui_glow.as_mut().unwrap().on_window_event(
            self.main_state.aurora_window.as_ref().unwrap().window(),
            event,
        );

        if event_response.repaint {
            self.main_state
                .aurora_window
                .as_ref()
                .unwrap()
                .window()
                .request_redraw();
        }
    }

    fn handle_cover_window_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::WindowEvent;

        let app_gl = self.app_gl.as_ref().unwrap();

        if let WindowEvent::Resized(physical_size) = event {
            self.cover.as_ref().unwrap().resize(app_gl, *physical_size);
        }

        let event_response = self.cover.as_mut().unwrap().on_window_event(event);

        if event_response.repaint {
            self.cover.as_ref().unwrap().request_redraw();
        }
    }
}

impl winit::application::ApplicationHandler<UserEvent> for AuroraRunner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.init_app(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                event_loop.exit();
                return;
            }
            WindowEvent::RedrawRequested => {
                self.redraw(event_loop, window_id);
                return;
            }
            _ => {}
        }

        let is_main = self.is_main_window(window_id);
        let is_cover = self.is_cover_window(window_id);

        if is_main {
            self.handle_main_window_event(event_loop, &event);
        } else if is_cover {
            self.handle_cover_window_event(&event);
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Redraw(delay) => {
                self.platform.repaint_delay = Some(delay);
                if delay.is_zero() {
                    if let Some(main) = &self.main_state.aurora_window {
                        main.window().request_redraw();
                    }
                    if let Some(cover) = &self.cover {
                        cover.request_redraw();
                    }
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                } else if let Some(repaint_after_instant) =
                    std::time::Instant::now().checked_add(delay)
                {
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                        repaint_after_instant,
                    ));
                } else {
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
                }
            }
            UserEvent::ImeEvent(event) => {
                self.main_state.pending_ime_events.push(event);
                if let Some(main) = &self.main_state.aurora_window {
                    main.window().request_redraw();
                }
            }
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = &cause {
            if let Some(main) = &self.main_state.aurora_window {
                main.window().request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // IME events arrive via background-thread callback → EventLoopProxy.
        // No periodic polling is needed.
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.cleanup();
    }
}
