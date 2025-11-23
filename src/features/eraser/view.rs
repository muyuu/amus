use crate::state::AppState;

pub struct EraserView;

impl EraserView {
    pub fn render(state: &mut AppState, ui: &mut egui::Ui) {
        Self::render_erase_button(state, ui);
    }

    fn render_erase_button(state: &mut AppState, ui: &mut egui::Ui) {
        if ui.button("Erase").clicked() {
            state.toggle_eraser_mode();
        }
    }
}