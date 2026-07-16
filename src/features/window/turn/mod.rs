mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{Slices, WaveAction};
use egui::Ui;
use view::TurnView;

pub struct TurnFeature;

impl TurnFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        let waves_len = match slices.game().game() {
            Some(game) => game.waves.len(),
            None => return Vec::new(),
        };

        let res = TurnView::render(slices, ui, waves_len);

        res.selected_wave_index
            .map(|index| AppAction::Wave(WaveAction::Select(index)))
            .into_iter()
            .collect()
    }
}
