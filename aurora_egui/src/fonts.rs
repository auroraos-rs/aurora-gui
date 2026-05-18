use aurora_services::FontSettings;
use egui::{FontData, FontDefinitions, FontFamily, FontId, TextStyle};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Load system fonts matching the Aurora OS font settings.
///
/// Returns `None` if no matching fonts were found on the system.
/// On success, the returned [`FontDefinitions`] includes the system fonts
/// as the highest-priority choice for `Proportional` (and a custom "Heading"
/// family when the heading font differs), with egui's built-in fonts kept
/// as fallbacks.
pub fn load_system_fonts(font_settings: &FontSettings) -> Option<FontDefinitions> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut font_defs = FontDefinitions::default();
    let mut loaded_any = false;

    // Load main / body font
    if let Some((name, data)) = query_font(&db, &font_settings.family) {
        font_defs
            .font_data
            .insert(name.clone(), Arc::new(data));
        let proportional = font_defs
            .families
            .entry(FontFamily::Proportional)
            .or_default();
        proportional.insert(0, name.clone());
        loaded_any = true;
    }

    // Load heading font when it differs from the body font
    if font_settings.family_heading != font_settings.family {
        if let Some((name, data)) = query_font(&db, &font_settings.family_heading) {
            font_defs
                .font_data
                .insert(name.clone(), Arc::new(data));
            let heading_family = FontFamily::Name("Heading".into());
            font_defs
                .families
                .insert(heading_family, vec![name.clone()]);
            loaded_any = true;
        }
    }

    if loaded_any {
        Some(font_defs)
    } else {
        None
    }
}

/// Build egui text-style size mappings from Aurora OS font settings.
///
/// The sizes from DConf are device-independent "silica pixels"; we use them
/// directly as point sizes because `init_app` already calls
/// `set_pixels_per_point(pixel_ratio)`, so egui will scale them correctly
/// for the device's physical display.
pub fn apply_font_settings(ctx: &egui::Context, font_settings: &FontSettings) {
    if let Some(font_defs) = load_system_fonts(font_settings) {
        ctx.set_fonts(font_defs);
    }
    let text_styles = build_text_styles(font_settings);
    ctx.global_style_mut(|style| style.text_styles = text_styles);
}

pub fn build_text_styles(font_settings: &FontSettings) -> BTreeMap<TextStyle, FontId> {
    let heading_family = if font_settings.family_heading != font_settings.family {
        FontFamily::Name("Heading".into())
    } else {
        FontFamily::Proportional
    };

    BTreeMap::from([
        (
            TextStyle::Small,
            FontId::new(font_settings.size_small as f32, FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(font_settings.size_medium as f32, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(font_settings.size_medium as f32, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(font_settings.size_large as f32, heading_family),
        ),
        (
            TextStyle::Monospace,
            FontId::new(font_settings.size_medium as f32, FontFamily::Monospace),
        ),
    ])
}

fn query_font(db: &fontdb::Database, family_name: &str) -> Option<(String, FontData)> {
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(family_name)],
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    };

    let id = db.query(&query)?;
    let face = db.face(id)?;

    let bytes = match &face.source {
        fontdb::Source::Binary(data) => data.as_ref().as_ref().to_vec(),
        fontdb::Source::File(path) => std::fs::read(path).ok()?,
        fontdb::Source::SharedFile(path, _) => std::fs::read(path).ok()?,
    };

    Some((family_name.to_string(), FontData::from_owned(bytes)))
}
