use super::Actions;

/// ウェーブ操作から発行されるアクション
#[derive(Debug, Clone)]
pub enum WaveAction {
    /// ウェーブを選択
    Select(usize),
}

impl Actions<'_> {
    pub fn handle_wave(&mut self, action: WaveAction) {
        match action {
            WaveAction::Select(index) => {
                self.state.select_wave(index);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Actions, AppState};

    #[test]
    fn select_updates_current_wave_index() {
        let mut state = AppState::new();
        state.create_game_from_setup();

        Actions::new(&mut state).handle_wave(WaveAction::Select(3));

        assert_eq!(state.slices().wave().current_wave_index(), 3);
    }
}
