use crate::state::AppState;

pub struct EraserView;

impl EraserView {
    pub fn render(_state: &AppState, ui: &mut egui::Ui) -> bool {
        ui.button("Erase").clicked()
    }
}
