mod view;

use crate::state::{Actions, AppState};
use egui::Ui;
use view::EraserView;

pub struct EraserFeature;

impl EraserFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let slices = state.slices();
        let eraser_actions = EraserView::render(&slices, ui);
        let mut actions = Actions::new(state);
        for action in eraser_actions {
            actions.handle_eraser(action);
        }
    }
}
