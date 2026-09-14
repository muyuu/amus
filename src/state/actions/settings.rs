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
    /// 録音に使う入力デバイスを設定（`None` はシステム既定）。
    /// 実際のハードウェア切り替えはアプリシェル（`AmusApp`）側で行うため、
    /// ここでは選択内容の保存のみを担う。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    SetInputDevice(Option<String>),
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
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            SettingsAction::SetInputDevice(name) => self.state.set_input_device_name(name),
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
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    fn set_input_device_updates_state() {
        let mut state = AppState::new();

        Actions::new(&mut state)
            .handle_settings(SettingsAction::SetInputDevice(Some("USB Mic".to_string())));
        assert_eq!(
            state.slices().ui().input_device_name(),
            Some("USB Mic".to_string())
        );

        Actions::new(&mut state).handle_settings(SettingsAction::SetInputDevice(None));
        assert_eq!(state.slices().ui().input_device_name(), None);
    }

    #[test]
    fn close_hides_settings() {
        let mut state = AppState::new();
        state.toggle_settings();

        Actions::new(&mut state).handle_settings(SettingsAction::Close);

        assert!(!state.show_settings());
    }
}
