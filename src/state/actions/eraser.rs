use super::Actions;

/// 消しゴム機能から発行されるアクション
#[derive(Debug, Clone)]
pub enum EraserAction {
    /// 消しゴムモードをトグル
    Toggle,
}

impl Actions<'_> {
    pub fn handle_eraser(&self, action: EraserAction) {
        match action {
            EraserAction::Toggle => {
                let current = self.state.erase_mode();
                self.state.set_erase_mode(!current);
                self.state.set_selected_player_id(None);
            }
        }
    }
}
