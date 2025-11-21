use crate::models::{Area, Color};
use crate::state::AppState;

pub struct SetupInteraction;

impl SetupInteraction {
    pub fn select_area(state: &mut AppState, area: Area) {
        state.setup_state.selected_area = area.clone();
    }

    pub fn adjust_player_count(state: &mut AppState, new_count: usize) {
        state.adjust_player_count(new_count);
    }

    pub fn update_player_name(state: &mut AppState, index: usize, name: String) {
        if index < state.setup_state.players.len() {
            state.setup_state.players[index].name = name;
        }
    }

    pub fn select_player_color(state: &mut AppState, index: usize, color: Color) {
        if index < state.setup_state.players.len() {
            state.setup_state.players[index].color = color;
        }
    }

    pub fn start_game(state: &mut AppState) {
        state.create_game_from_setup();
    }

    pub fn cancel(state: &mut AppState) {
        state.cancel_setup();
    }
}
