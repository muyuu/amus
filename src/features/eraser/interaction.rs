use crate::state::AppState;

pub struct EraserInteraction;

impl EraserInteraction {
    pub fn toggle_eraser_mode(state: &mut AppState) {
        let current = state.erase_mode();
        state.set_erase_mode(!current);
        state.set_selected_user_id(None);
    }
}
