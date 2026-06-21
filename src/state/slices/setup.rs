use crate::models::player::Player;
use crate::models::Area;
use crate::state::app_data::AppData;

/// セットアップ状態の読み取り専用アクセスを提供
pub struct SetupSlice<'a> {
    data: &'a AppData,
}

impl<'a> SetupSlice<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// 選択中のエリア
    pub fn selected_area(&self) -> &Area {
        &self.data.setup_state.selected_area
    }

    /// プレイヤー人数
    pub fn player_count(&self) -> usize {
        self.data.setup_state.player_count
    }

    /// セットアップ中のプレイヤー一覧
    pub fn players(&self) -> &[Player] {
        &self.data.setup_state.players
    }
}
