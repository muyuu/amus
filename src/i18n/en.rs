use std::collections::HashMap;
use super::keys::*;
use super::words::en::EnglishWords as W;

pub fn get_translations() -> HashMap<&'static str, String> {
    let mut translations = HashMap::new();

    // アプリケーション
    translations.insert(APP_TITLE, "Among Us Assistant Tool".to_string());

    // メニュー (using word dictionary)
    translations.insert(MENU_FILE, W::FILE.to_string());
    translations.insert(MENU_NEW_GAME, format!("{} {}", W::NEW, W::GAME));
    translations.insert(MENU_LOAD_GAME, format!("{} {}", W::LOAD, W::GAME));
    translations.insert(MENU_SAVE_GAME, format!("{} {}", W::SAVE, W::GAME));
    translations.insert(MENU_LANGUAGE, W::LANGUAGE.to_string());

    // サイドバー (using word dictionary)
    translations.insert(SIDEBAR_TURN_MANAGEMENT, format!("{} {}", W::TURN, W::MANAGEMENT));
    translations.insert(SIDEBAR_ADD_NEW_TURN, format!("{} {} {}", W::ADD, W::NEW, W::TURN));
    translations.insert(SIDEBAR_PLAYERS, W::PLAYER.to_string());
    translations.insert(SIDEBAR_START_GAME_PROMPT, format!("Please {} a new {}", W::START.to_lowercase(), W::GAME.to_lowercase()));
    translations.insert(SIDEBAR_TURN_PREFIX, W::TURN.to_string());
    translations.insert(SIDEBAR_CURRENT_TURN_PREFIX, format!("▶ {}", W::TURN));

    // セットアップダイアログ (using word dictionary)
    translations.insert(SETUP_TITLE, format!("{} {}", W::GAME, W::SETTINGS));
    translations.insert(SETUP_AREA_SELECTION, format!("{} {}", W::AREA, W::SELECTION));
    translations.insert(SETUP_PLAYER_SETTINGS, format!("{} {}", W::PLAYER, W::SETTINGS));
    translations.insert(SETUP_PLAYER_COUNT, format!("{} {}:", W::PLAYER, W::COUNT));
    translations.insert(SETUP_PLAYER_CONFIG, format!("{} {}:", W::PLAYER, W::CONFIG));
    translations.insert(SETUP_CONFIG_NOTE, "(In development: Default settings available)".to_string());
    translations.insert(SETUP_START_GAME, format!("{} {}", W::START, W::GAME));
    translations.insert(SETUP_CANCEL, W::CANCEL.to_string());

    // メインコンテンツ (using word dictionary)
    translations.insert(MAIN_DRAWING_MODE, format!("{} {}:", W::DRAWING, W::MODE));
    translations.insert(MAIN_DRAWING_NONE, W::NONE.to_string());
    translations.insert(MAIN_DRAWING_CLICK_LINE, format!("{} to {}", W::CLICK, W::LINE));
    translations.insert(MAIN_DRAWING_FREEHAND, W::FREEHAND.to_string());
    translations.insert(MAIN_DISCUSSION_INFO, format!("{} {} {}", W::DISCUSSION, W::TURN, W::INFORMATION));
    translations.insert(MAIN_KILLED_PLAYER, format!("{} {}:", W::KILLED, W::PLAYER));
    translations.insert(MAIN_KILL_LOCATION, format!("Kill {} (Testimony):", W::LOCATION));
    translations.insert(MAIN_LOCATION_UNSET, W::UNSET.to_string());
    translations.insert(MAIN_SET_LOCATION_BTN, format!("{} on map to {}", W::CLICK, W::SET.to_lowercase()));
    translations.insert(MAIN_LOCATION_NOTE, "(In development: Right-click on map planned)".to_string());
    translations.insert(MAIN_NOTES, format!("{}:", W::NOTES));
    translations.insert(MAIN_NO_SELECTION, W::NONE.to_string());
    translations.insert(MAIN_SELECT_PROMPT, "Please select".to_string());
    translations.insert(MAIN_STATUS_ALIVE, W::ALIVE.to_string());
    translations.insert(MAIN_STATUS_DEAD, W::DEAD.to_string());
    translations.insert(MAIN_WELCOME_TITLE, "Welcome to Among Us Assistant Tool".to_string());
    translations.insert(MAIN_WELCOME_MESSAGE, format!("Please {} a new {}", W::START.to_lowercase(), W::GAME.to_lowercase()));

    translations
}