mod constants;
mod view;

use constants::PlayerInfoConstants;
use view::PlayerListView;

use crate::app_action::AppAction;
use crate::state::AppState;
use egui::Ui;

/// プレイヤーリスト機能
pub struct PlayerListFeature;

impl PlayerListFeature {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<AppAction> {
        let slices = state.slices();
        PlayerListView::render(&slices, ui)
            .into_iter()
            .map(AppAction::Player)
            .collect()
    }
}
