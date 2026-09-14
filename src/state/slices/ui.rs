use crate::models::location::DraggingLocation;
use crate::models::player::PlayerId;
use crate::state::app_data::AppData;
use crate::state::SettingsTab;

/// UI状態の読み取り専用アクセスを提供
pub struct UiSlice<'a> {
    data: &'a AppData,
}

impl<'a> UiSlice<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// デバッグビューの表示状態
    pub fn show_debug_view(&self) -> bool {
        self.data.show_debug_view
    }

    /// 消しゴムモードの状態
    pub fn erase_mode(&self) -> bool {
        self.data.erase_mode
    }

    /// ドラッグ中のプレイヤーID
    pub fn dragging_player_id(&self) -> Option<PlayerId> {
        self.data.dragging_player_id
    }

    /// 指定プレイヤーがドラッグ中かどうか
    pub fn is_dragging_player(&self, player_id: PlayerId) -> bool {
        self.data.dragging_player_id == Some(player_id)
    }

    /// ドラッグ中の位置情報
    pub fn dragging_location(&self) -> Option<DraggingLocation> {
        self.data.dragging_location
    }

    /// 設定モーダルで選択中のタブ
    pub fn settings_tab(&self) -> SettingsTab {
        self.data.settings_tab
    }

    /// 軌跡描画の線の太さ
    pub fn route_line_width(&self) -> f32 {
        self.data.route_line_width
    }

    /// UI 拡大率。`None` はネイティブ DPI 追従。
    pub fn ui_scale(&self) -> Option<f32> {
        self.data.ui_scale
    }
}
