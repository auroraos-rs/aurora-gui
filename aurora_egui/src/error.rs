use std::ffi::NulError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("failed to create OpenGL context: {0}")]
    GlContextCreation(String),

    #[error("failed to create window surface: {0}")]
    SurfaceCreation(String),

    #[error("failed to create window: {0}")]
    WindowCreation(String),

    #[error("failed to get window handle: {0}")]
    WindowHandle(String),

    #[error("failed to finalize window: {0}")]
    WindowFinalize(String),

    #[error("no suitable GL configuration found")]
    NoGlConfig,

    #[error("failed to make GL context current: {0}")]
    MakeCurrent(String),

    #[error("failed to set swap interval: {0}")]
    SwapInterval(String),

    #[error("failed to create C string: {0}")]
    NulError(#[from] NulError),

    #[error("event loop error: {0}")]
    EventLoop(String),

    #[error("Aurora services error: {0}")]
    AuroraServices(#[from] aurora_services::AuroraError),
}

pub type Result<T> = std::result::Result<T, AppError>;
