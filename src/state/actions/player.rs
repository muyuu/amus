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
    pub fn handle_player(&mut self, action: PlayerAction) {
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
    fn select_sets_selected_and_clears_erase_mode() {
        let (mut state, id) = state_with_player();
        state.set_erase_mode(true);

        Actions::new(&mut state).handle_player(PlayerAction::Select(id));

        assert_eq!(state.selected_player_id(), Some(id));
        assert!(!state.erase_mode());
    }

    #[test]
    fn start_drag_sets_dragging_and_selected() {
        let (mut state, id) = state_with_player();

        Actions::new(&mut state).handle_player(PlayerAction::StartDrag(id));

        assert_eq!(state.dragging_player_id(), Some(id));
        assert_eq!(state.selected_player_id(), Some(id));
    }

    #[test]
    fn stop_drag_clears_dragging() {
        let (mut state, id) = state_with_player();
        Actions::new(&mut state).handle_player(PlayerAction::StartDrag(id));

        Actions::new(&mut state).handle_player(PlayerAction::StopDrag);

        assert!(state.dragging_player_id().is_none());
    }
}
