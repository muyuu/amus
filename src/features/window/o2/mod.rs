mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{Slices, WindowAction};
use egui::Ui;
use view::O2View;

pub struct O2Feature;

impl O2Feature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        let res = O2View::render(slices, ui);

        res.click_player
            .map(|player| AppAction::Window(WindowAction::ToggleO2(player.id)))
            .into_iter()
            .collect()
    }
}
