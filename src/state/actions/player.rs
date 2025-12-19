use super::Actions;
use crate::models::player::PlayerId;

/// プレイヤーリストViewから発行されるアクション
#[derive(Debug, Clone)]
pub enum PlayerAction {
    /// プレイヤーを選択
    Select(PlayerId),
    /// ドラッグ開始
    StartDrag(PlayerId),
    /// ドラッグ終了
    StopDrag,
}

impl Actions<'_> {
    /// PlayerActionを処理
    pub fn handle_player(&self, action: PlayerAction) {
        match action {
            PlayerAction::Select(id) => {
                self.state.set_selected_player_id(Some(id));
                self.state.set_erase_mode(false);
            }
            PlayerAction::StartDrag(id) => {
                self.state.set_selected_player_id(Some(id));
                self.state.set_erase_mode(false);
                self.state.set_dragging_player_id(Some(id));
            }
            PlayerAction::StopDrag => {
                self.state.set_dragging_player_id(None);
            }
        }
    }
}
