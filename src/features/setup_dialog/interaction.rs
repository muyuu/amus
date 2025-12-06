use crate::models::player::PlayerId;
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

    pub fn update_player_name(state: &mut AppState, id: PlayerId, name: String) {
        state.update_player_name(id, name);
    }

    pub fn update_player_color(state: &mut AppState, id: PlayerId, color: Color) {
        // 現在の色と同じ色なら何もしない
        if let Some(current_color) = state.player_color(id) {
            if current_color == color {
                return;
            }
        }

        state.update_player(id, color);
    }

    pub fn reset_selected_player_color(state: &mut AppState) {
        state.reset_selected_player_color();
    }

    pub fn start_game(state: &mut AppState) {
        state.reset_game();
        state.create_game_from_setup();
    }

    pub fn cancel(state: &mut AppState) {
        state.cancel_setup();
    }
}
