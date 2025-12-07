use crate::{i18n::keys::*, log_debug, state::AppState};

use super::view::ReactorViewResult;

pub struct ReactorInteraction;

impl ReactorInteraction {
    pub fn handle(state: &mut AppState, res: &ReactorViewResult) {
        let clicked_player = match &res.click_player {
            Some(p) => p,
            None => return,
        };

        state.toggle_reactor(clicked_player.id);

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
