use super::view::TurnViewResult;
use crate::state::AppState;

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
