use crate::models::{Area, PlayerId};

use super::AppState;

// UI状態管理関連
impl AppState {
    pub fn show_setup_dialog(&self) -> bool {
        self.data.show_setup_dialog
    }

    pub fn set_erase_mode(&mut self, mode: bool) {
        self.data.erase_mode = mode;
    }

    pub fn set_selected_player_id(&mut self, player_id: Option<PlayerId>) {
        self.data.selected_player_id = player_id;
    }

    pub fn set_selected_area(&mut self, area: Area) {
        self.data.setup_state.selected_area = area;
    }

    pub fn toggle_setup_dialog(&mut self) {
        self.data.show_setup_dialog = !self.data.show_setup_dialog;
    }

    #[cfg(debug_assertions)]
    pub fn toggle_debug_view(&mut self) {
        self.data.show_debug_view = !self.data.show_debug_view;
    }
}
