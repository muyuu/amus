mod constants;
mod view;

use crate::state::{Actions, AppState, WindowAction};
use egui::Ui;
use view::TurnView;

pub struct TurnFeature;

impl TurnFeature {
    pub fn render(state: &AppState, ui: &mut Ui) {
        let game = match state.game() {
            Some(game) => game,
            None => return,
        };

        let res = TurnView::render(state, ui, game.waves.len());

        if let Some(index) = res.selected_wave_index {
            let actions = Actions::new(state);
            actions.handle_window(WindowAction::SelectWave(index));
        }
    }
}
