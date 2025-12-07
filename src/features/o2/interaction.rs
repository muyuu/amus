use crate::{i18n::keys::*, log_debug, state::AppState};

use super::view::O2ViewResult;

pub struct O2Interaction;

impl O2Interaction {
    pub fn handle(state: &mut AppState, res: &O2ViewResult) {
        let clicked_player = match &res.click_player {
            Some(p) => p,
            None => return,
        };

        state.toggle_o2(clicked_player.id);

        log_debug!(
            "O2Interaction",
            format!(
                "{}を切り替え: {:?} -> {:?}",
                state.t(SABOTAGE_O2).to_string(),
                clicked_player.name,
                !clicked_player.resolved_lights,
            )
        );
    }
}
