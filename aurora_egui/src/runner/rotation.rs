use egui_rotate::Rotation as EguiRotation;

/// Map a winit `Transform` to an `egui-rotate` `Rotation`.
pub fn transform_to_rotation(transform: &winit::event::Transform) -> EguiRotation {
    match transform {
        winit::event::Transform::Normal => EguiRotation::None,
        winit::event::Transform::_90 => EguiRotation::CW90,
        winit::event::Transform::_180 => EguiRotation::CW180,
        winit::event::Transform::_270 => EguiRotation::CW270,
        other => {
            log::warn!(
                "Unsupported transform variant for egui-rotate: {:?}. Treating as no rotation.",
                other
            );
            EguiRotation::None
        }
    }
}
