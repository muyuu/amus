use crate::{features::player_info::view::PlayerInfoResult, state::AppState};

pub struct PlayerInfoInteraction;

impl PlayerInfoInteraction {
    pub fn handle(state: &mut AppState, res: &PlayerInfoResult) {
        if res.any_name_double_clicked {
            Self::player_double_clicked(state, res);
        }

        if res.any_name_changed {
            Self::player_name_changed(state, res);
        }

        if res.any_lost_focus {
            Self::player_name_lost_focus(state, res);
        }

        if res.any_done_button_clicked {
            Self::player_done_button_clicked(state, res);
        }
    }

    pub fn player_double_clicked(state: &mut AppState, res: &PlayerInfoResult) {
        if let Some(player_id) = res.double_clicked_player_id {
            state.toggle_player_name_editing(player_id);
        }
    }

    pub fn player_name_changed(state: &mut AppState, res: &PlayerInfoResult) {
        if let Some((new_name, user_id)) = &res.changed_new_name {
            state.update_player_name(*user_id, new_name.clone());
        }
    }

    pub fn player_name_lost_focus(state: &mut AppState, res: &PlayerInfoResult) {
        if let Some(user_id) = res.lost_focus_player_id {
            state.toggle_player_name_editing(user_id);
        }
    }

    pub fn player_done_button_clicked(state: &mut AppState, res: &PlayerInfoResult) {
        let game = match state.game() {
            Some(g) => g,
            None => return,
        };

        let player_id = match res.done_button_clicked_player_id {
            Some(id) => id,
            None => return,
        };

        let player = match game.players.get(player_id) {
            Some(p) => p,
            None => return,
        };

        state.update_player_button(player_id, !player.done_button);
    }
}
