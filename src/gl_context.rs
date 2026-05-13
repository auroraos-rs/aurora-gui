use crate::error::{AppError, Result};
use aurora_app::window::MainWindow;
use std::num::NonZeroU32;
use winit::raw_window_handle::HasWindowHandle;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WINDOW_TITLE: &str = "egui_glow example";

/// Manages the OpenGL context, surface, and window for the application.
pub struct GlutinWindowContext {
    window: MainWindow,
    pub gl_context: glutin::context::PossiblyCurrentContext,
    pub gl_display: glutin::display::Display,
    pub gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

struct RawGlContext {
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl RawGlContext {
    unsafe fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_attributes: winit::window::WindowAttributes,
    ) -> Result<(winit::window::Window, Self)> {
        use glutin::context::NotCurrentGlContext;
        use glutin::display::GetGlDisplay;
        use glutin::display::GlDisplay;
        use glutin::prelude::GlSurface;

        let winit_window_builder = window_attributes;

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false);

        let (mut window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPreference::FallbackEgl)
            .with_window_attributes(Some(winit_window_builder.clone()))
            .build(event_loop, config_template_builder, |mut config_iterator| {
                config_iterator
                    .next()
                    .expect("failed to find a matching configuration for creating glutin config")
            })
            .map_err(|e| AppError::GlContextCreation(e.to_string()))?;

        let gl_display = gl_config.display();

        let raw_window_handle = window
            .as_ref()
            .map(|w| w.window_handle().map(|h| h.as_raw()))
            .transpose()
            .map_err(|e| AppError::WindowHandle(e.to_string()))?;

        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);

        // SAFETY: Creating a GL context requires unsafe. We have a valid display and config.
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                })
                .map_err(|e| AppError::GlContextCreation(e.to_string()))?
        };

        let window = window.take().unwrap_or_else(|| {
            // SAFETY: finalize_window is safe when called with valid event loop and config.
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });

        let (width, height): (u32, u32) = window.inner_size().into();
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);

        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .map_err(|e| AppError::WindowHandle(e.to_string()))?
                        .as_raw(),
                    width,
                    height,
                );

        // SAFETY: Creating a window surface requires unsafe. We have a valid display and surface attributes.
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .map_err(|e| AppError::SurfaceCreation(e.to_string()))?
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .map_err(|e| AppError::MakeCurrent(e.to_string()))?;

        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(NonZeroU32::MIN),
            )
            .map_err(|e| AppError::SwapInterval(e.to_string()))?;

        Ok((
            window,
            Self {
                gl_context,
                gl_display,
                gl_surface,
            },
        ))
    }
}

impl GlutinWindowContext {
    /// Creates a new GL context and window.
    ///
    /// # Safety
    /// This function contains unsafe blocks required by glutin for context creation
    /// and surface creation. These are safe when called from the main thread with
    /// a valid event loop.
    pub unsafe fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        let window_attributes = winit::window::WindowAttributes::default()
            .with_resizable(true)
            .with_inner_size(winit::dpi::LogicalSize {
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
            })
            .with_title(WINDOW_TITLE)
            .with_visible(false);
        unsafe { Self::new_with_attributes(event_loop, window_attributes) }
    }

    pub unsafe fn new_with_attributes(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_attributes: winit::window::WindowAttributes,
    ) -> Result<Self> {
        let (window, raw) = unsafe { RawGlContext::new(event_loop, window_attributes)? };

        let window = MainWindow::new(window);
        window.set_statusbar_visible(true);
        window.set_background_visible(true);

        Ok(Self {
            window,
            gl_context: raw.gl_context,
            gl_display: raw.gl_display,
            gl_surface: raw.gl_surface,
        })
    }

    pub fn window(&self) -> &winit::window::Window {
        self.window.window()
    }

    pub fn main_window(&self) -> &aurora_app::window::MainWindow {
        &self.window
    }

    pub fn main_window_mut(&mut self) -> &mut aurora_app::window::MainWindow {
        &mut self.window
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;
        let width = NonZeroU32::new(physical_size.width.max(1)).unwrap();
        let height = NonZeroU32::new(physical_size.height.max(1)).unwrap();
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn swap_buffers(&self) -> Result<()> {
        use glutin::surface::GlSurface;
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .map_err(|e| AppError::GlContextCreation(e.to_string()))
    }

    pub fn get_proc_address(&self,
        addr: &std::ffi::CStr,
    ) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay;
        self.gl_display.get_proc_address(addr)
    }
}

/// GL context for the cover window.
pub struct CoverGlContext {
    pub window: winit::window::Window,
    pub gl_context: glutin::context::PossiblyCurrentContext,
    pub gl_display: glutin::display::Display,
    pub gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl CoverGlContext {
    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;
        let width = NonZeroU32::new(physical_size.width.max(1)).unwrap();
        let height = NonZeroU32::new(physical_size.height.max(1)).unwrap();
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn swap_buffers(&self) -> Result<()> {
        use glutin::surface::GlSurface;
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .map_err(|e| AppError::GlContextCreation(e.to_string()))
    }

    pub fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay;
        self.gl_display.get_proc_address(addr)
    }
}

/// Creates the display and GL context for the main window.
///
/// # Safety
/// Must be called from the main thread with a valid event loop.
pub unsafe fn create_display(
    event_loop: &winit::event_loop::ActiveEventLoop,
) -> Result<(GlutinWindowContext, glow::Context)> {
    // SAFETY: Caller must ensure this is called from main thread with valid event loop.
    let glutin_window_context = unsafe { GlutinWindowContext::new(event_loop)? };

    // SAFETY: from_loader_function is unsafe because the caller must ensure
    // the loader function is valid for the lifetime of the Context.
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s).expect("failed to construct C string");
            glutin_window_context.get_proc_address(&s)
        })
    };

    Ok((glutin_window_context, gl))
}

/// Creates the display and GL context for the cover window.
///
/// # Safety
/// Must be called from the main thread with a valid event loop.
pub unsafe fn create_cover_display(
    event_loop: &winit::event_loop::ActiveEventLoop,
) -> Result<(CoverGlContext, glow::Context)> {
    let window_attributes = winit::window::WindowAttributes::default()
        .with_visible(false);

    // SAFETY: Caller must ensure this is called from main thread with valid event loop.
    let (window, raw) = unsafe { RawGlContext::new(event_loop, window_attributes)? };

    let ctx = CoverGlContext {
        window,
        gl_context: raw.gl_context,
        gl_display: raw.gl_display,
        gl_surface: raw.gl_surface,
    };

    // SAFETY: from_loader_function is unsafe because the caller must ensure
    // the loader function is valid for the lifetime of the Context.
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s).expect("failed to construct C string");
            ctx.get_proc_address(&s)
        })
    };

    Ok((ctx, gl))
}
