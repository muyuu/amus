use crate::egui::*;
use crate::{i18n::keys::ERASER_BUTTON, state::AppState};

pub struct EraserView;

impl EraserView {
    pub fn render(state: &AppState, ui: &mut Ui) -> bool {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(40.0, 40.0), Sense::click_and_drag());

        ui.painter().rect_filled(rect, 4.0, Color32::PLACEHOLDER);

        if state.erase_mode() {
            ui.painter()
                .rect_stroke(rect, 4.0, (2.0, Color32::WHITE), StrokeKind::Inside);
        }
        ui.label(RichText::new(state.t(ERASER_BUTTON)).color(Color32::WHITE));
        response.clicked()
    }
}
