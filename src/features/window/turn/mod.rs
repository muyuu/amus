mod constants;
mod view;

use crate::state::{Actions, AppState, WindowAction};
use egui::Ui;
use view::TurnView;

pub struct TurnFeature;

impl TurnFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let slices = state.slices();
        let game_slice = slices.game();
        let waves_len = match game_slice.game() {
            Some(game) => game.waves.len(),
            None => return,
        };

        let res = TurnView::render(&slices, ui, waves_len);

        if let Some(index) = res.selected_wave_index {
            let mut actions = Actions::new(state);
            actions.handle_window(WindowAction::SelectWave(index));
        }
    }
}
