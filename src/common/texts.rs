use crate::i18n::keys::*;
use crate::state::AppState;

/// 複数のUIコンポーネント間で共通して使用されるテキスト
pub struct CommonTexts {
    // ボタン系
    pub button_new_game: String,
    pub button_start_game: String,
    pub button_cancel: String,
}

impl CommonTexts {
    pub fn get(state: &AppState) -> Self {
        Self {
            // ボタン系
            button_new_game: state.t(MENU_NEW_GAME).to_string(),
            button_start_game: state.t(SETUP_START_GAME).to_string(),
            button_cancel: state.t(SETUP_CANCEL).to_string(),
        }
    }
}
