use crate::constants::AppConstants;
use crate::models::{Area, PlayerId};

use super::AppState;

// UI状態管理関連
impl AppState {
    pub fn show_setup_dialog(&self) -> bool {
        self.data.show_setup_dialog
    }

    /// UI 拡大率。`None` はネイティブ DPI 追従（明示的な上書きをしない）、
    /// `Some(x)` はユーザー指定倍率。
    pub fn ui_scale(&self) -> Option<f32> {
        self.data.ui_scale
    }

    /// UI 拡大率をユーザー指定値に設定する。値は有効範囲へクランプされる。
    pub fn set_ui_scale(&mut self, scale: f32) {
        self.data.ui_scale =
            Some(scale.clamp(AppConstants::UI_SCALE_MIN, AppConstants::UI_SCALE_MAX));
    }

    /// UI 拡大率をネイティブ DPI 追従（自動）に戻す。
    pub fn reset_ui_scale(&mut self) {
        self.data.ui_scale = None;
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

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::constants::AppConstants;

    #[test]
    fn ui_scale_defaults_to_auto() {
        // 既定は None（ネイティブ DPI に追従し、明示的な上書きをしない）
        assert_eq!(AppState::new().ui_scale(), None);
    }

    #[test]
    fn set_ui_scale_clamps_to_range() {
        let mut state = AppState::new();

        state.set_ui_scale(2.0);
        assert_eq!(state.ui_scale(), Some(2.0));

        state.set_ui_scale(99.0);
        assert_eq!(state.ui_scale(), Some(AppConstants::UI_SCALE_MAX));

        state.set_ui_scale(0.01);
        assert_eq!(state.ui_scale(), Some(AppConstants::UI_SCALE_MIN));
    }

    #[test]
    fn reset_ui_scale_returns_to_auto() {
        let mut state = AppState::new();
        state.set_ui_scale(2.0);
        state.reset_ui_scale();
        assert_eq!(state.ui_scale(), None);
    }
}
