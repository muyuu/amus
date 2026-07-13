use super::Actions;
use crate::models::player::PlayerId;
use crate::models::Sabotage;

/// ウィンドウ系機能から発行されるアクション
#[derive(Debug, Clone)]
pub enum WindowAction {
    /// サボタージュの解除状態をトグル
    ToggleSabotage(Sabotage, PlayerId),
    /// ウェーブを選択
    SelectWave(usize),
}

impl Actions<'_> {
    pub fn handle_window(&mut self, action: WindowAction) {
        match action {
            WindowAction::ToggleSabotage(kind, player_id) => {
                self.state.toggle_sabotage(kind, player_id);
            }
            WindowAction::SelectWave(index) => {
                self.state.select_wave(index);
            }
        }
    }
}
