use crate::models::{Area, Game, Wave};

use super::AppState;

// ゲーム管理関連
impl AppState {
    pub fn start_new_game(&mut self) {
        self.data.show_setup_dialog = true;
    }

    pub fn create_game_from_setup(&mut self) {
        let area = self.data.setup_state.selected_area.clone();
        let player_count = self.data.setup_state.player_count;
        let players = self
            .data
            .setup_state
            .players
            .iter()
            .take(player_count)
            .cloned()
            .collect();

        let mut new_game = Game::new(area, players);
        // とりあえず20ウェーブ作成
        for _ in 0..20 {
            new_game.waves.push(Wave::default());
        }
        self.data.game = Some(new_game);
        self.data.current_wave_index = 0;
        self.data.show_setup_dialog = false;
    }

    pub fn cancel_setup(&mut self) {
        self.data.show_setup_dialog = false;
    }

    pub fn reset_game(&mut self) {
        // setup_state は変えなくて良いケースが多いはずなので保持
        let setup_state = self.data.setup_state.clone();
        self.data = crate::state::app_data::AppData::default();
        self.data.setup_state = setup_state;
    }

    pub fn select_wave(&mut self, index: usize) {
        self.data.current_wave_index = index;
    }

    pub fn game(&self) -> Option<&Game> {
        self.data.game.as_ref()
    }

    pub fn area(&self) -> Option<Area> {
        self.game().map(|game| game.area.clone())
    }
}
