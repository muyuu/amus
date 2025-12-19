use super::Actions;

/// ゲーム管理機能から発行されるアクション
#[derive(Debug, Clone)]
pub enum GameAction {
    /// 新しいゲームを開始（セットアップダイアログを表示）
    StartNewGame,
}

impl Actions<'_> {
    pub fn handle_game(&self, action: GameAction) {
        match action {
            GameAction::StartNewGame => {
                self.state.start_new_game();
            }
        }
    }
}
