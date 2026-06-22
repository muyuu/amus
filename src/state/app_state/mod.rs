mod drag_drop;
mod game;
mod player;
mod storage;
mod ui;
mod wave;

// TextKey はネイティブのウィンドウタイトル取得（t）でのみ使う。
#[cfg(not(target_arch = "wasm32"))]
use crate::i18n::keys::TextKey;
use crate::state::app_data::AppData;
use crate::state::slices::Slices;

/// アプリケーション状態へのアクセスを提供する構造体
#[derive(Default)]
pub struct AppState {
    pub(super) data: AppData,
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

    // ネイティブのウィンドウタイトル設定だけが使う（描画側は Slices::t を使う）。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn t(&self, key: TextKey) -> String {
        self.data.translator.t(key).to_string()
    }
}
