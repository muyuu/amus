pub mod interaction;
pub mod view;

pub use interaction::EraserInteraction;

pub struct EraserFeature;
impl EraserFeature {
    pub fn render(state: &mut crate::state::AppState, ui: &mut egui::Ui) {
        let toggle_erase_clicked = view::EraserView::render(state, ui);

        if toggle_erase_clicked {
            EraserInteraction::toggle_eraser_mode(state);
        }
    }
}
