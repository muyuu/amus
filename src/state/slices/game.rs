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
    pub fn game(&self) -> Option<&'a Game> {
        self.data.game.as_ref()
    }

    /// ゲームの世代番号。作り直すたびに進む。
    ///
    /// 前回見たときと違えば別のゲームになっている。内容が同じでも区別できる。
    ///
    /// 読むのが voice_memo（ネイティブ専用）だけなので、それを含まないビルドでは未使用。
    #[cfg_attr(
        not(all(not(target_arch = "wasm32"), feature = "voice_memo")),
        allow(dead_code)
    )]
    pub fn generation(&self) -> u64 {
        self.data.game_generation
    }

    /// ゲームが存在するかどうか
    pub fn has_game(&self) -> bool {
        self.data.game.is_some()
    }

    /// 現在のエリアへの参照を取得
    pub fn area(&self) -> Option<&'a Area> {
        self.data.game.as_ref().map(|g| &g.area)
    }
}
