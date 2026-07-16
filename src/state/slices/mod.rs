mod game;
mod player;
mod setup;
mod ui;
mod wave;

pub use game::GameSlice;
pub use player::PlayerSlice;
pub use setup::SetupSlice;
pub use ui::UiSlice;
pub use wave::WaveSlice;

use super::app_data::AppData;

/// 読み取り専用のStateアクセスを提供する構造体
///
/// Viewはこの構造体を通じてAppDataにアクセスする。
/// 書き込みはActionsを通じて行う。
pub struct Slices<'a> {
    data: &'a AppData,
}

impl<'a> Slices<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// 翻訳されたテキストを取得
    pub fn t(&self, key: crate::i18n::keys::TextKey) -> String {
        self.data.translator.t(key).to_string()
    }

    /// 翻訳器への参照（同じ翻訳器で複数回 `t` を引きたい場合に使う）
    // 現状は voice_memo（ネイティブ専用）だけが使う。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn translator(&self) -> &'a crate::i18n::Translator {
        &self.data.translator
    }

    /// ゲーム関連の読み取り専用アクセス
    pub fn game(&self) -> GameSlice<'a> {
        GameSlice::new(self.data)
    }

    /// プレイヤー関連の読み取り専用アクセス
    pub fn player(&self) -> PlayerSlice<'a> {
        PlayerSlice::new(self.data)
    }

    /// ウェーブ関連の読み取り専用アクセス
    pub fn wave(&self) -> WaveSlice<'a> {
        WaveSlice::new(self.data)
    }

    /// UI状態の読み取り専用アクセス
    pub fn ui(&self) -> UiSlice<'a> {
        UiSlice::new(self.data)
    }

    /// セットアップ状態の読み取り専用アクセス
    pub fn setup(&self) -> SetupSlice<'a> {
        SetupSlice::new(self.data)
    }
}
