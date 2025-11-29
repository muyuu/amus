use crate::models::{Area, Color};
use crate::state::AppState;

pub struct SetupInteraction;

impl SetupInteraction {
    pub fn select_area(state: &mut AppState, area: Area) {
        state.set_selected_area(area);
    }

    pub fn adjust_player_count(state: &mut AppState, new_count: usize) {
        state.adjust_player_count(new_count);
    }

    pub fn update_player_name(state: &mut AppState, index: usize, name: String) {
        if index < state.setup_state().players.len() {
            state.update_player_name(index, name);
        }
    }

    pub fn update_player_color(state: &mut AppState, index: usize, color: Color) {
        if index < state.setup_state().players.len() {
            state.update_player_color(index, color);
        }
    }

    pub fn start_game(state: &mut AppState) {
        state.reset_game();
        state.create_game_from_setup();
    }

    pub fn cancel(state: &mut AppState) {
        state.cancel_setup();
    }
}
