use crate::models::{Area, Game};
use crate::state::app_data::AppData;

/// ゲーム関連の読み取り専用アクセスを提供
pub struct GameSlice<'a> {
    data: &'a AppData,
}

impl<'a> GameSlice<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// 現在のゲームへの参照を取得
    pub fn game(&self) -> Option<&Game> {
        self.data.game.as_ref()
    }

    /// ゲームが存在するかどうか
    pub fn has_game(&self) -> bool {
        self.data.game.is_some()
    }

    /// 現在のエリアへの参照を取得
    pub fn area(&self) -> Option<&Area> {
        self.data.game.as_ref().map(|g| &g.area)
    }
}
