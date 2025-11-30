use crate::{features::user_info::view::UserInfoResult, state::AppState};

pub struct UserInfoInteraction;

impl UserInfoInteraction {
    pub fn handle(state: &mut AppState, res: &UserInfoResult) {
        if res.any_name_double_clicked {
            Self::user_double_clicked(state, res);
        }

        if res.any_name_changed {
            Self::user_name_changed(state, res);
        }

        if res.any_lost_focus {
            Self::user_name_lost_focus(state, res);
        }
    }
    pub fn user_double_clicked(state: &mut AppState, res: &UserInfoResult) {
        if let Some(user_id) = res.double_clicked_user_id {
            state.toggle_user_name_editing(user_id);
        }
    }

    pub fn user_name_changed(state: &mut AppState, res: &UserInfoResult) {
        if let Some((new_name, user_id)) = &res.changed_new_name {
            state.update_player_name(*user_id, new_name.clone());
        }
    }

    pub fn user_name_lost_focus(state: &mut AppState, res: &UserInfoResult) {
        if let Some(user_id) = res.lost_focus_user_id {
            state.toggle_user_name_editing(user_id);
        }
    }
}
