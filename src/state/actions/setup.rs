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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Actions, AppState};

    #[test]
    fn select_area_updates_setup_state() {
        let mut state = AppState::new();

        Actions::new(&mut state).handle_setup(SetupAction::SelectArea(Area::Polus));

        assert_eq!(state.slices().setup().selected_area(), &Area::Polus);
    }

    #[test]
    fn adjust_player_count_changes_count() {
        let mut state = AppState::new();

        Actions::new(&mut state).handle_setup(SetupAction::AdjustPlayerCount(15));

        assert_eq!(state.slices().setup().player_count(), 15);
    }

    #[test]
    fn start_game_creates_game_and_closes_dialog() {
        let mut state = AppState::new();
        state.start_new_game();

        Actions::new(&mut state).handle_setup(SetupAction::StartGame);

        assert!(state.slices().game().game().is_some());
        assert!(!state.show_setup_dialog());
    }

    #[test]
    fn cancel_closes_dialog() {
        let mut state = AppState::new();
        state.start_new_game();

        Actions::new(&mut state).handle_setup(SetupAction::Cancel);

        assert!(!state.show_setup_dialog());
    }
}
