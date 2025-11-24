use crate::state::AppState;

pub struct EraserView;

impl EraserView {
    pub fn render(_state: &AppState, ui: &mut egui::Ui) -> bool {
        let toggle_erase_mode = ui.button("Erase").clicked();

        toggle_erase_mode
    }
}
