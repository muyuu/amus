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
        // ViewにはSlices（読み取り専用）を渡す
        let slices = state.slices();
        let player_actions = PlayerListView::render(&slices, ui);

        // Actionsで状態を更新
        let mut actions = Actions::new(state);
        for action in player_actions {
            actions.handle_player(action);
        }
    }
}
