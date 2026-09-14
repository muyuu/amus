use super::Actions;
use crate::state::SettingsTab;

/// 設定モーダルから発行されるアクション
#[derive(Debug, Clone)]
pub enum SettingsAction {
    /// タブを切り替え
    SelectTab(SettingsTab),
    /// UI 拡大率を指定値に設定
    SetUiScale(f32),
    /// UI 拡大率をネイティブ DPI 追従に戻す
    ResetUiScale,
    /// 軌跡描画の線の太さを設定
    SetRouteLineWidth(f32),
    /// デバッグビューの表示切り替え
    #[cfg(debug_assertions)]
    ToggleDebugView,
    /// モーダルを閉じる
    Close,
}

impl Actions<'_> {
    pub fn handle_settings(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::SelectTab(tab) => self.state.set_settings_tab(tab),
            SettingsAction::SetUiScale(scale) => self.state.set_ui_scale(scale),
            SettingsAction::ResetUiScale => self.state.reset_ui_scale(),
            SettingsAction::SetRouteLineWidth(width) => self.state.set_route_line_width(width),
            #[cfg(debug_assertions)]
            SettingsAction::ToggleDebugView => self.state.toggle_debug_view(),
            SettingsAction::Close => self.state.close_settings(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Actions, AppState};

    #[test]
    fn select_tab_updates_settings_tab() {
        let mut state = AppState::new();

        Actions::new(&mut state).handle_settings(SettingsAction::SelectTab(SettingsTab::Display));

        assert_eq!(state.slices().ui().settings_tab(), SettingsTab::Display);
    }

    #[test]
    fn set_route_line_width_updates_state() {
        let mut state = AppState::new();

        Actions::new(&mut state).handle_settings(SettingsAction::SetRouteLineWidth(10.0));

        assert_eq!(state.slices().ui().route_line_width(), 10.0);
    }

    #[test]
    fn close_hides_settings() {
        let mut state = AppState::new();
        state.toggle_settings();

        Actions::new(&mut state).handle_settings(SettingsAction::Close);

        assert!(!state.show_settings());
    }
}
