mod drag_drop;
mod game;
mod player;
mod storage;
mod ui;
mod wave;

use crate::state::app_data::AppData;
use crate::state::slices::Slices;

/// アプリケーション状態へのアクセスを提供する構造体
pub struct AppState {
    pub(super) data: AppData,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: AppData::default(),
        }
    }
}

// 基本的なコンストラクタとデータアクセス
impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 読み取り専用のSlicesを取得
    ///
    /// ViewはこのSlicesを通じてデータにアクセスする。
    /// 書き込みはActionsを通じて行う。
    pub fn slices(&self) -> Slices<'_> {
        Slices::new(&self.data)
    }

    pub fn t(&self, key: &str) -> String {
        self.data.translator.t(key).to_string()
    }
}
