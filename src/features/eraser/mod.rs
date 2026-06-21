mod view;

use crate::app_action::AppAction;
use crate::state::Slices;
use egui::Ui;
use view::EraserView;

pub struct EraserFeature;

impl EraserFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        EraserView::render(slices, ui)
            .into_iter()
            .map(AppAction::Eraser)
            .collect()
    }
}
