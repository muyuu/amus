use super::Actions;
use crate::models::player::PlayerId;

/// ウィンドウ系機能から発行されるアクション
#[derive(Debug, Clone)]
pub enum WindowAction {
    /// 通信妨害の解除状態をトグル
    ToggleComms(PlayerId),
    /// 停電の解除状態をトグル
    ToggleLights(PlayerId),
    /// 酸素妨害の解除状態をトグル
    ToggleO2(PlayerId),
    /// リアクター妨害の解除状態をトグル
    ToggleReactor(PlayerId),
    /// ウェーブを選択
    SelectWave(usize),
}

impl Actions<'_> {
    pub fn handle_window(&mut self, action: WindowAction) {
        match action {
            WindowAction::ToggleComms(player_id) => {
                self.state.toggle_comms(player_id);
            }
            WindowAction::ToggleLights(player_id) => {
                self.state.toggle_lights(player_id);
            }
            WindowAction::ToggleO2(player_id) => {
                self.state.toggle_o2(player_id);
            }
            WindowAction::ToggleReactor(player_id) => {
                self.state.toggle_reactor(player_id);
            }
            WindowAction::SelectWave(index) => {
                self.state.select_wave(index);
            }
        }
    }
}
