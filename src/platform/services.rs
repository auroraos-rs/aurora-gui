/// Stub for Aurora settings service.
/// In Phase 2, this will integrate with the real aurora_services crate.

#[derive(Debug, Clone)]
pub struct FontSettings {
    pub family: String,
    pub size_small: u32,
    pub size_medium: u32,
    pub size_large: u32,
}

pub struct SettingsService;

impl SettingsService {
    pub fn new() -> crate::error::Result<Self> {
        Ok(Self)
    }

    pub fn get_statusbar_height(&self) -> crate::error::Result<u32> {
        Ok(41)
    }

    pub fn get_pixel_ratio(&self) -> crate::error::Result<f32> {
        Ok(1.5)
    }

    pub fn get_font_settings(&self) -> crate::error::Result<FontSettings> {
        Ok(FontSettings {
            family: "Sailfish Sans".to_string(),
            size_small: 12,
            size_medium: 16,
            size_large: 20,
        })
    }
}
