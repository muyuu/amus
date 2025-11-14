use crate::game::state::AppState;
use crate::i18n::keys::*;

/// 複数のUIコンポーネント間で共通して使用されるテキスト
pub struct CommonTexts {
    // ボタン系
    pub button_new_game: String,
    pub button_start_game: String,
    pub button_cancel: String,

    // ステータス系
    pub status_alive: String,
    pub status_dead: String,

    // 汎用ラベル
    pub label_notes: String,
    pub label_turn_prefix: String,

    // 選択系
    pub select_no_selection: String,
    pub select_prompt: String,
}

impl CommonTexts {
    pub fn get(state: &AppState) -> Self {
        Self {
            // ボタン系
            button_new_game: state.t(MENU_NEW_GAME).to_string(),
            button_start_game: state.t(SETUP_START_GAME).to_string(),
            button_cancel: state.t(SETUP_CANCEL).to_string(),

            // ステータス系
            status_alive: state.t(MAIN_STATUS_ALIVE).to_string(),
            status_dead: state.t(MAIN_STATUS_DEAD).to_string(),

            // 汎用ラベル
            label_notes: state.t(MAIN_NOTES).to_string(),
            label_turn_prefix: state.t(SIDEBAR_TURN_PREFIX).to_string(),

            // 選択系
            select_no_selection: state.t(MAIN_NO_SELECTION).to_string(),
            select_prompt: state.t(MAIN_SELECT_PROMPT).to_string(),
        }
    }
}