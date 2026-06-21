mod view;

use crate::app_action::AppAction;
use crate::state::AppState;
use egui::Ui;
use view::EraserView;

pub struct EraserFeature;

impl EraserFeature {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<AppAction> {
        let slices = state.slices();
        EraserView::render(&slices, ui)
            .into_iter()
            .map(AppAction::Eraser)
            .collect()
    }
}
