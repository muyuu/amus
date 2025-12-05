pub mod interaction;
pub mod view;

use egui::*;
pub use interaction::EraserInteraction;
pub use view::EraserView;

use crate::state::AppState;

pub struct EraserFeature;
impl EraserFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let toggle_erase_clicked = EraserView::render(state, ui);

        if toggle_erase_clicked {
            EraserInteraction::toggle_eraser_mode(state);
        }
    }
}
