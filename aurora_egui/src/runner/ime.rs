use aurora_app::{InputMethodEvent, InputMethodKey};

/// Drain pending Maliit events into the given egui `RawInput`.
pub fn process_maliit_events(
    pending_events: &mut Vec<aurora_app::InputMethodEvent>,
    keyboard_area: &mut Option<egui::Rect>,
    main_egui_glow: &Option<egui_glow::EguiGlow>,
    raw_input: &mut egui::RawInput,
) {
    for event in pending_events.drain(..) {
        match event {
            InputMethodEvent::AreaChanged(x, y, width, height) => {
                *keyboard_area = Some(egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(width as f32, height as f32),
                ));
            }
            InputMethodEvent::ImInitiatedHide => {
                *keyboard_area = None;
                if let Some(main) = main_egui_glow {
                    let focused_id = main.egui_ctx.memory(|mem| mem.focused());
                    if let Some(id) = focused_id {
                        main.egui_ctx.memory_mut(|mem| mem.surrender_focus(id));
                    }
                }
            }
            other => {
                if let Some(egui_event) = convert_maliit_event(other) {
                    raw_input.events.push(egui_event);
                }
            }
        }
    }
}

fn convert_maliit_event(event: InputMethodEvent) -> Option<egui::Event> {
    match event {
        InputMethodEvent::Text(text) => Some(egui::Event::Text(text)),
        InputMethodEvent::Key {
            key: InputMethodKey::Enter,
            pressed,
        } => Some(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }),
        InputMethodEvent::Key {
            key: InputMethodKey::Backspace,
            pressed,
        } => Some(egui::Event::Key {
            key: egui::Key::Backspace,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }),
        InputMethodEvent::AreaChanged(_, _, _, _) => None,
        InputMethodEvent::ActivationLost => None,
        InputMethodEvent::ImInitiatedHide => None,
    }
}
