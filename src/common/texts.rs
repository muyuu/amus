use crate::i18n::keys::*;
use crate::state::Slices;

/// 複数のUIコンポーネント間で共通して使用されるテキスト
pub struct CommonTexts {
    // ボタン系
    pub button_new_game: String,
    pub button_start_game: String,
    pub button_cancel: String,
    pub button_close: String,
}

impl CommonTexts {
    pub fn get(slices: &Slices<'_>) -> Self {
        Self {
            // ボタン系
            button_new_game: slices.t(MENU_NEW_GAME),
            button_start_game: slices.t(SETUP_START_GAME),
            button_cancel: slices.t(SETUP_CANCEL),
            button_close: slices.t(SETTINGS_CLOSE),
        }
    }
}
