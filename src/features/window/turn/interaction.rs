use crate::{state::AppState};
use super::view::TurnViewResult;

pub struct TurnInteraction;

impl TurnInteraction {
    pub fn handle(state: &mut AppState, res: &TurnViewResult) {
        let selected_wave_index = match res.selected_wave_index {
            Some(index) => index,
            None => return,
        };

        state.select_wave(selected_wave_index);
    }
}
