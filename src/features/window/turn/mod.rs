mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{AppState, WindowAction};
use egui::Ui;
use view::TurnView;

pub struct TurnFeature;

impl TurnFeature {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<AppAction> {
        let slices = state.slices();
        let game_slice = slices.game();
        let waves_len = match game_slice.game() {
            Some(game) => game.waves.len(),
            None => return Vec::new(),
        };

        let res = TurnView::render(&slices, ui, waves_len);

        res.selected_wave_index
            .map(|index| AppAction::Window(WindowAction::SelectWave(index)))
            .into_iter()
            .collect()
    }
}
