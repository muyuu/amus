use crate::{i18n::keys::ERASER_BUTTON, state::AppState};
use crate::egui::*;

pub struct EraserView;

impl EraserView {
    pub fn render(state: &AppState, ui: &mut egui::Ui) -> bool {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(40.0, 40.0),
            Sense::click_and_drag(),
        );

        ui.painter().rect_filled(rect, 4.0, Color32::PLACEHOLDER);

        if state.erase_mode() {
            ui.painter().rect_stroke(rect, 4.0, (2.0, Color32::WHITE));
        }
        ui.label(state.t(ERASER_BUTTON));
        response.clicked()
    }
}
