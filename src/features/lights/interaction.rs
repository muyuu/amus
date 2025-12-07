use crate::{i18n::keys::SABOTAGE_LIGHTS, log_debug, state::AppState};

pub use super::view::LightsViewResult;

pub struct LightsInteraction;

impl LightsInteraction {
    pub fn handle(state: &mut AppState, res: &LightsViewResult) {
        let clicked_player = match &res.click_player {
            Some(p) => p,
            None => return,
        };

        state.toggle_lights(clicked_player.id);

        log_debug!(
            "LightsInteraction",
            format!(
                "{}を切り替え: {:?} -> {:?}",
                state.t(SABOTAGE_LIGHTS).to_string(),
                clicked_player.name,
                !clicked_player.resolved_lights,
            )
        );
    }
}
