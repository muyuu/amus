mod view;

use crate::state::{Actions, AppState};
use egui::Ui;
use view::EraserView;

pub struct EraserFeature;

impl EraserFeature {
    pub fn render(state: &AppState, ui: &mut Ui) {
        let eraser_actions = EraserView::render(state, ui);
        let actions = Actions::new(state);
        for action in eraser_actions {
            actions.handle_eraser(action);
        }
    }
}
