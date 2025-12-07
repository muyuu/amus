use super::view::CommsViewResult;
use crate::{log_debug, state::AppState};

pub struct CommsInteraction;

impl CommsInteraction {
    pub fn handle(state: &mut AppState, res: &CommsViewResult) {
        let clicked_player = match &res.click_player {
            Some(p) => p,
            None => return,
        };

        // TODO: ここに処理を書く
        state.toggle_comms(clicked_player.id);

        log_debug!(
            "CommsInteraction",
            format!(
                "通信妨害を切り替え: {:?} -> {:?}",
                clicked_player.name, !clicked_player.resolved_comms,
            )
        );
    }
}
