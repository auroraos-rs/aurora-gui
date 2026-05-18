use egui_rotate::Rotation as EguiRotation;

/// Map a winit `Transform` to an `egui-rotate` `Rotation`.
pub fn transform_to_rotation(transform: &winit::event::Transform) -> Option<EguiRotation> {
    match transform {
        winit::event::Transform::Normal => None,
        winit::event::Transform::_90 => Some(EguiRotation::CW90),
        winit::event::Transform::_180 => Some(EguiRotation::CW180),
        winit::event::Transform::_270 => Some(EguiRotation::CW270),
        other => {
            log::warn!(
                "Unsupported transform variant for egui-rotate: {:?}. Treating as no rotation.",
                other
            );
            None
        }
    }
}
