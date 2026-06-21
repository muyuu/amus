use super::Actions;
use crate::models::color::Color;
use crate::models::player::PlayerId;

/// プレイヤー情報パネルから発行されるアクション
#[derive(Debug, Clone)]
pub enum PlayerInfoAction {
    /// 名前の編集を開始
    StartEditingName(PlayerId),
    /// 名前を更新
    UpdateName(PlayerId, String),
    /// 名前の編集を終了
    StopEditingName(PlayerId),
    /// 完了ボタンをトグル
    ToggleDoneButton(PlayerId),
    /// 色を変更
    ChangeColor(PlayerId, Color),
}

impl Actions<'_> {
    pub fn handle_player_info(&mut self, action: PlayerInfoAction) {
        match action {
            PlayerInfoAction::StartEditingName(player_id) => {
                self.state.toggle_player_name_editing(player_id);
            }
            PlayerInfoAction::UpdateName(player_id, name) => {
                self.state.update_player_name(player_id, name);
            }
            PlayerInfoAction::StopEditingName(player_id) => {
                self.state.toggle_player_name_editing(player_id);
            }
            PlayerInfoAction::ToggleDoneButton(player_id) => {
                let game = match self.state.game() {
                    Some(g) => g,
                    None => return,
                };
                let player = match game.players.iter().find(|p| p.id == player_id) {
                    Some(p) => p,
                    None => return,
                };
                self.state
                    .update_player_button(player_id, !player.done_button);
            }
            PlayerInfoAction::ChangeColor(player_id, color) => {
                if let Err(e) = self.state.try_update_player_color(player_id, color) {
                    eprintln!("色の変更に失敗: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Actions, AppState};

    fn state_with_player() -> (AppState, PlayerId) {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let id = state.players().unwrap()[0].id;
        (state, id)
    }

    #[test]
    fn update_name_changes_player_name() {
        let (mut state, id) = state_with_player();

        Actions::new(&mut state)
            .handle_player_info(PlayerInfoAction::UpdateName(id, "きいろ".to_string()));

        assert_eq!(state.player(id).unwrap().name, "きいろ");
    }

    #[test]
    fn toggle_done_button_flips_flag() {
        let (mut state, id) = state_with_player();
        assert!(!state.player(id).unwrap().done_button);

        Actions::new(&mut state).handle_player_info(PlayerInfoAction::ToggleDoneButton(id));

        assert!(state.player(id).unwrap().done_button);
    }

    #[test]
    fn start_editing_name_marks_player_editing() {
        let (mut state, id) = state_with_player();

        Actions::new(&mut state).handle_player_info(PlayerInfoAction::StartEditingName(id));

        assert!(state.player_name_editing(id));
    }
}
