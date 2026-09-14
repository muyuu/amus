use crate::constants::AppConstants;
use crate::models::{Area, PlayerId};
use crate::state::SettingsTab;

use super::AppState;

// UI状態管理関連
impl AppState {
    pub fn show_setup_dialog(&self) -> bool {
        self.data.show_setup_dialog
    }

    pub fn show_settings(&self) -> bool {
        self.data.show_settings
    }

    /// 設定モーダルの表示を切り替える。
    pub fn toggle_settings(&mut self) {
        self.data.show_settings = !self.data.show_settings;
    }

    /// 設定モーダルを閉じる。
    pub fn close_settings(&mut self) {
        self.data.show_settings = false;
    }

    pub fn set_settings_tab(&mut self, tab: SettingsTab) {
        self.data.settings_tab = tab;
    }

    /// 軌跡描画の線の太さを設定する。値は有効範囲へクランプされる。
    pub fn set_route_line_width(&mut self, width: f32) {
        self.data.route_line_width = width.clamp(
            AppConstants::ROUTE_LINE_WIDTH_MIN,
            AppConstants::ROUTE_LINE_WIDTH_MAX,
        );
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

    #[test]
    fn set_route_line_width_clamps_to_range() {
        let mut state = AppState::new();

        state.set_route_line_width(10.0);
        assert_eq!(state.slices().ui().route_line_width(), 10.0);

        state.set_route_line_width(99.0);
        assert_eq!(
            state.slices().ui().route_line_width(),
            AppConstants::ROUTE_LINE_WIDTH_MAX
        );

        state.set_route_line_width(0.0);
        assert_eq!(
            state.slices().ui().route_line_width(),
            AppConstants::ROUTE_LINE_WIDTH_MIN
        );
    }

    #[test]
    fn toggle_settings_flips_visibility() {
        let mut state = AppState::new();

        assert!(!state.show_settings());
        state.toggle_settings();
        assert!(state.show_settings());
        state.close_settings();
        assert!(!state.show_settings());
    }
}
