mod constants;
mod view;

use crate::state::{Actions, AppState, WindowAction};
use egui::Ui;
use view::O2View;

pub struct O2Feature;

impl O2Feature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let slices = state.slices();
        let res = O2View::render(&slices, ui);

        if let Some(player) = res.click_player {
            let mut actions = Actions::new(state);
            actions.handle_window(WindowAction::ToggleO2(player.id));
        }
    }
}
