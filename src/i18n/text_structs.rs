use crate::game::state::AppState;
use crate::i18n::keys::*;

// アプリケーション共通のテキスト
pub struct AppTexts {
    pub file_menu: String,
    pub new_game: String,
    pub load_game: String,
    pub save_game: String,
    pub language_menu: String,
}

impl AppTexts {
    pub fn get(state: &AppState) -> Self {
        Self {
            file_menu: state.t(MENU_FILE).to_string(),
            new_game: state.t(MENU_NEW_GAME).to_string(),
            load_game: state.t(MENU_LOAD_GAME).to_string(),
            save_game: state.t(MENU_SAVE_GAME).to_string(),
            language_menu: state.t(MENU_LANGUAGE).to_string(),
        }
    }
}

// サイドバー用のテキスト
pub struct SidebarTexts {
    pub turn_management: String,
    pub add_new_turn: String,
    pub players: String,
    pub start_game_prompt: String,
    pub turn_prefix: String,
    pub current_turn_prefix: String,
    pub new_game: String,
}

impl SidebarTexts {
    pub fn get(state: &AppState) -> Self {
        Self {
            turn_management: state.t(SIDEBAR_TURN_MANAGEMENT).to_string(),
            add_new_turn: state.t(SIDEBAR_ADD_NEW_TURN).to_string(),
            players: state.t(SIDEBAR_PLAYERS).to_string(),
            start_game_prompt: state.t(SIDEBAR_START_GAME_PROMPT).to_string(),
            turn_prefix: state.t(SIDEBAR_TURN_PREFIX).to_string(),
            current_turn_prefix: state.t(SIDEBAR_CURRENT_TURN_PREFIX).to_string(),
            new_game: state.t(MENU_NEW_GAME).to_string(),
        }
    }
}

// セットアップダイアログ用のテキスト
pub struct SetupTexts {
    pub title: String,
    pub area_selection: String,
    pub player_settings: String,
    pub player_count: String,
    pub player_config: String,
    pub config_note: String,
    pub start_game: String,
    pub cancel: String,
}

impl SetupTexts {
    pub fn get(state: &AppState) -> Self {
        Self {
            title: state.t(SETUP_TITLE).to_string(),
            area_selection: state.t(SETUP_AREA_SELECTION).to_string(),
            player_settings: state.t(SETUP_PLAYER_SETTINGS).to_string(),
            player_count: state.t(SETUP_PLAYER_COUNT).to_string(),
            player_config: state.t(SETUP_PLAYER_CONFIG).to_string(),
            config_note: state.t(SETUP_CONFIG_NOTE).to_string(),
            start_game: state.t(SETUP_START_GAME).to_string(),
            cancel: state.t(SETUP_CANCEL).to_string(),
        }
    }
}

// メインコンテンツ用のテキスト
pub struct MainContentTexts {
    pub turn_prefix: String,
    pub drawing_mode: String,
    pub drawing_none: String,
    pub drawing_click_line: String,
    pub drawing_freehand: String,
    pub discussion_info: String,
    pub killed_player: String,
    pub no_selection: String,
    pub select_prompt: String,
    pub status_alive: String,
    pub status_dead: String,
    pub kill_location: String,
    pub location_unset: String,
    pub set_location_btn: String,
    pub location_note: String,
    pub notes: String,
    pub welcome_title: String,
    pub welcome_message: String,
    pub new_game: String,
}

impl MainContentTexts {
    pub fn get(state: &AppState) -> Self {
        Self {
            turn_prefix: state.t(SIDEBAR_TURN_PREFIX).to_string(),
            drawing_mode: state.t(MAIN_DRAWING_MODE).to_string(),
            drawing_none: state.t(MAIN_DRAWING_NONE).to_string(),
            drawing_click_line: state.t(MAIN_DRAWING_CLICK_LINE).to_string(),
            drawing_freehand: state.t(MAIN_DRAWING_FREEHAND).to_string(),
            discussion_info: state.t(MAIN_DISCUSSION_INFO).to_string(),
            killed_player: state.t(MAIN_KILLED_PLAYER).to_string(),
            no_selection: state.t(MAIN_NO_SELECTION).to_string(),
            select_prompt: state.t(MAIN_SELECT_PROMPT).to_string(),
            status_alive: state.t(MAIN_STATUS_ALIVE).to_string(),
            status_dead: state.t(MAIN_STATUS_DEAD).to_string(),
            kill_location: state.t(MAIN_KILL_LOCATION).to_string(),
            location_unset: state.t(MAIN_LOCATION_UNSET).to_string(),
            set_location_btn: state.t(MAIN_SET_LOCATION_BTN).to_string(),
            location_note: state.t(MAIN_LOCATION_NOTE).to_string(),
            notes: state.t(MAIN_NOTES).to_string(),
            welcome_title: state.t(MAIN_WELCOME_TITLE).to_string(),
            welcome_message: state.t(MAIN_WELCOME_MESSAGE).to_string(),
            new_game: state.t(MENU_NEW_GAME).to_string(),
        }
    }
}