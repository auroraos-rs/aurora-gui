use egui::Ui;

/// Persistent UI state for the application.
#[derive(Debug, Clone)]
pub struct UiState {
    pub clear_color: [f32; 3],
    pub show_side_panel: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            clear_color: crate::config::AppConfig::CLEAR_COLOR,
            show_side_panel: true,
        }
    }
}

/// Draws the UI and returns `true` if the user requested to quit.
pub fn draw_ui(ui: &mut Ui, state: &mut UiState) -> bool {
    let mut quit = false;

    egui::Panel::left("my_side_panel").show_inside(ui, |ui: &mut Ui| {
        ui.heading("Hello World!");
        if ui.button("Quit").clicked() {
            quit = true;
        }

        ui.color_edit_button_rgb(state.clear_color.as_mut().try_into().unwrap());
    });

    quit
}
