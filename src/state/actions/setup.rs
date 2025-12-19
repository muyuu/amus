use super::Actions;
use crate::models::player::PlayerId;
use crate::models::{Area, Color};

/// セットアップダイアログから発行されるアクション
#[derive(Debug, Clone)]
pub enum SetupAction {
    /// エリアを選択
    SelectArea(Area),
    /// プレイヤー数を調整
    AdjustPlayerCount(usize),
    /// プレイヤーの名前と色を更新
    UpdatePlayer(PlayerId, String, Color),
    /// ゲームを開始
    StartGame,
    /// セットアップをキャンセル
    Cancel,
}

impl Actions<'_> {
    pub fn handle_setup(&mut self, action: SetupAction) {
        match action {
            SetupAction::SelectArea(area) => {
                self.state.set_selected_area(area);
            }
            SetupAction::AdjustPlayerCount(count) => {
                self.state.adjust_player_count(count);
            }
            SetupAction::UpdatePlayer(id, name, color) => {
                self.state.update_player_name(id, name);
                self.state.force_update_player_color(id, color);
            }
            SetupAction::StartGame => {
                self.state.reset_game();
                self.state.create_game_from_setup();
            }
            SetupAction::Cancel => {
                self.state.cancel_setup();
            }
        }
    }
}
