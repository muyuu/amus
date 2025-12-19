mod constants;
mod view;

use constants::PlayerInfoConstants;
use view::PlayerListView;

use crate::state::{Actions, AppState};
use egui::Ui;

/// プレイヤーリスト機能
///
/// View（描画）とAction処理を統合
pub struct PlayerListFeature;

impl PlayerListFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let player_actions = PlayerListView::render(state, ui);
        let mut actions = Actions::new(state);
        for action in player_actions {
            actions.handle_player(action);
        }
    }
}
