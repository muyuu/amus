mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::AppState;
use egui::*;
use view::PlayerInfoView;

pub struct PlayerInfoFeature;

impl PlayerInfoFeature {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<AppAction> {
        let slices = state.slices();
        PlayerInfoView::render(&slices, ui)
            .into_iter()
            .map(AppAction::PlayerInfo)
            .collect()
    }
}
