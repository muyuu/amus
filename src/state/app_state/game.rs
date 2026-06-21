use crate::models::{Game, Wave};

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Area;

    #[test]
    fn start_new_game_opens_setup_dialog() {
        let mut state = AppState::new();
        assert!(!state.show_setup_dialog());

        state.start_new_game();

        assert!(state.show_setup_dialog());
    }

    #[test]
    fn create_game_from_setup_builds_game_and_closes_dialog() {
        let mut state = AppState::new();
        state.start_new_game();
        let expected_players = state.slices().setup().player_count();

        state.create_game_from_setup();

        let game = state.slices().game().game().expect("ゲームが生成される");
        assert_eq!(game.players.len(), expected_players);
        // create_game_from_setup は固定で 20 wave 作る
        assert_eq!(game.waves.len(), 20);
        assert_eq!(state.slices().wave().current_wave_index(), 0);
        assert!(!state.show_setup_dialog());
    }

    #[test]
    fn reset_game_clears_game_but_keeps_setup_state() {
        let mut state = AppState::new();
        state.set_selected_area(Area::Polus);
        state.create_game_from_setup();
        assert!(state.slices().game().game().is_some());

        state.reset_game();

        assert!(state.slices().game().game().is_none());
        // setup_state は保持される
        assert_eq!(state.slices().setup().selected_area(), &Area::Polus);
    }

    #[test]
    fn select_wave_updates_current_index() {
        let mut state = AppState::new();
        state.create_game_from_setup();

        state.select_wave(3);

        assert_eq!(state.slices().wave().current_wave_index(), 3);
    }

    #[test]
    fn cancel_setup_closes_dialog() {
        let mut state = AppState::new();
        state.start_new_game();

        state.cancel_setup();

        assert!(!state.show_setup_dialog());
    }
}
