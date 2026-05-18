use crate::error::{AppError, Result};
use glutin::context::NotCurrentGlContext;
use glutin::display::GlDisplay;
use glutin::display::GetGlDisplay;
use glutin::prelude::GlSurface;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::raw_window_handle::HasWindowHandle;

/// Shared GL state for the entire application.
///
/// Both the main window and the cover window use the same GL context,
/// display connection, and `glow::Context`. Only the per-window surfaces
/// are separate.
pub struct AppGlContext {
    pub gl_context: glutin::context::PossiblyCurrentContext,
    pub gl_display: glutin::display::Display,
    pub gl_config: glutin::config::Config,
    pub glow: Arc<glow::Context>,
}

/// Per-window GL surface.
pub struct WindowSurface {
    pub gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl WindowSurface {
    pub fn resize(
        &self,
        gl_context: &glutin::context::PossiblyCurrentContext,
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) {
        use glutin::surface::GlSurface;
        let width = NonZeroU32::new(physical_size.width.max(1)).unwrap();
        let height = NonZeroU32::new(physical_size.height.max(1)).unwrap();
        self.gl_surface.resize(gl_context, width, height);
    }

    pub fn swap_buffers(
        &self,
        gl_context: &glutin::context::PossiblyCurrentContext,
    ) -> Result<()> {
        use glutin::surface::GlSurface;
        self.gl_surface
            .swap_buffers(gl_context)
            .map_err(|e| AppError::GlContextCreation(e.to_string()))
    }
}

// ------------------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------------------

fn build_config_template() -> glutin::config::ConfigTemplateBuilder {
    glutin::config::ConfigTemplateBuilder::new()
        .prefer_hardware_accelerated(None)
        .with_depth_size(0)
        .with_stencil_size(0)
        .with_transparency(false)
}

/// Create the main winit window, GL display, config, and surface.
fn create_main_window_raw(
    event_loop: &winit::event_loop::ActiveEventLoop,
    window_attributes: winit::window::WindowAttributes,
) -> Result<(
    winit::window::Window,
    glutin::display::Display,
    glutin::config::Config,
    glutin::surface::Surface<glutin::surface::WindowSurface>,
)> {

    let config_template = build_config_template();

    let (mut window, gl_config) = glutin_winit::DisplayBuilder::new()
        .with_preference(glutin_winit::ApiPreference::FallbackEgl)
        .with_window_attributes(Some(window_attributes.clone()))
        .build(
            event_loop,
            config_template,
            |mut configs| {
                configs
                    .next()
                    .expect("failed to find a matching configuration for creating glutin config")
            },
        )
        .map_err(|e| AppError::GlContextCreation(e.to_string()))?;

    let gl_display = gl_config.display();

    let window = window.take().unwrap_or_else(|| {
        // SAFETY: finalize_window is safe when called with valid event loop and config.
        glutin_winit::finalize_window(event_loop, window_attributes.clone(), &gl_config)
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

    Ok((window, gl_display, gl_config, gl_surface))
}

/// Create a GL context from a display + config, make it current with the given surface.
fn create_and_make_current(
    gl_display: &glutin::display::Display,
    gl_config: &glutin::config::Config,
    gl_surface: &glutin::surface::Surface<glutin::surface::WindowSurface>,
    raw_window_handle: Option<winit::raw_window_handle::RawWindowHandle>,
) -> Result<glutin::context::PossiblyCurrentContext> {

    let context_attributes =
        glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
    let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::Gles(None))
        .build(raw_window_handle);

    // SAFETY: Creating a GL context requires unsafe. We have a valid display and config.
    let not_current = unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .or_else(|_| gl_display.create_context(gl_config, &fallback_context_attributes))
            .map_err(|e| AppError::GlContextCreation(e.to_string()))?
    };

    let current = not_current
        .make_current(gl_surface)
        .map_err(|e| AppError::MakeCurrent(e.to_string()))?;

    gl_surface
        .set_swap_interval(
            &current,
            glutin::surface::SwapInterval::Wait(NonZeroU32::MIN),
        )
        .map_err(|e| AppError::SwapInterval(e.to_string()))?;

    Ok(current)
}

// ------------------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------------------

/// Creates the main window, its GL surface, and a shared application GL context.
///
/// The returned `Window` should be wrapped in `aurora_app::window::MainWindow`.
/// The returned `WindowSurface` should be stored for rendering.
///
/// # Safety
/// Must be called from the main thread with a valid event loop.
pub unsafe fn create_main_window(
    event_loop: &winit::event_loop::ActiveEventLoop,
    window_attributes: winit::window::WindowAttributes,
) -> Result<(winit::window::Window, WindowSurface, AppGlContext)> {
    let (window, gl_display, gl_config, gl_surface) =
        create_main_window_raw(event_loop, window_attributes)?;

    let raw_window_handle = window
        .window_handle()
        .map_err(|e| AppError::WindowHandle(e.to_string()))?
        .as_raw();

    let gl_context =
        create_and_make_current(&gl_display, &gl_config, &gl_surface, Some(raw_window_handle))?;

    // SAFETY: from_loader_function is unsafe because the caller must ensure
    // the loader function is valid for the lifetime of the Context.
    let glow = Arc::new(unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s).expect("failed to construct C string");
            gl_display.get_proc_address(&s)
        })
    });

    let surface = WindowSurface { gl_surface };
    let app_gl = AppGlContext {
        gl_context,
        gl_display,
        gl_config,
        glow,
    };

    Ok((window, surface, app_gl))
}

/// Creates a cover window and its GL surface, reusing the application's shared GL context.
///
/// The returned `Window` should be wrapped in `aurora_app::window::CoverWindow`.
/// The returned `WindowSurface` should be stored for rendering.
///
/// The shared GL context can be switched to this surface via:
/// `app_gl.gl_context.make_current(&surface.gl_surface)`.
///
/// # Safety
/// Must be called from the main thread with a valid event loop.
pub unsafe fn create_cover_surface(
    event_loop: &winit::event_loop::ActiveEventLoop,
    app_gl: &AppGlContext,
) -> Result<(winit::window::Window, WindowSurface)> {

    let window_attributes = winit::window::WindowAttributes::default();

    let window = event_loop
        .create_window(window_attributes)
        .map_err(|e| AppError::WindowCreation(e.to_string()))?;

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
        app_gl
            .gl_display
            .create_window_surface(&app_gl.gl_config, &surface_attributes)
            .map_err(|e| AppError::SurfaceCreation(e.to_string()))?
    };

    Ok((window, WindowSurface { gl_surface }))
}
