use crate::models::location::DraggingLocation;
use crate::models::player::PlayerId;
use crate::state::app_data::AppData;

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

    /// セットアップダイアログの表示状態
    pub fn show_setup_dialog(&self) -> bool {
        self.data.show_setup_dialog
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

    /// 位置がドラッグ中かどうか
    pub fn is_dragging_location(&self) -> bool {
        self.data.dragging_location.is_some()
    }
}
