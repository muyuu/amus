mod constants;
mod view;

use crate::state::{Actions, AppState};
use egui::*;
use view::PlayerInfoView;

pub struct PlayerInfoFeature;

impl PlayerInfoFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let slices = state.slices();
        let player_info_actions = PlayerInfoView::render(&slices, ui);
        let mut actions = Actions::new(state);
        for action in player_info_actions {
            actions.handle_player_info(action);
        }
    }
}
