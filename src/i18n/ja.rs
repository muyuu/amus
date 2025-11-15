use super::keys::*;
use super::words::ja::JapaneseWords as W;
use std::collections::HashMap;

pub fn get_translations() -> HashMap<&'static str, String> {
    let mut translations = HashMap::new();

    // アプリケーション
    translations.insert(APP_TITLE, "Among Us 補助ツール".to_string());

    // メニュー（単語辞書を使用）
    translations.insert(MENU_FILE, W::FILE.to_string());
    translations.insert(MENU_NEW_GAME, format!("{}{}", W::NEW, W::GAME)); // 複数単語の組み合わせ
    translations.insert(MENU_LOAD_GAME, format!("{}を{}", W::GAME, W::LOAD));
    translations.insert(MENU_SAVE_GAME, format!("{}を{}", W::GAME, W::SAVE));
    translations.insert(MENU_LANGUAGE, W::LANGUAGE.to_string());

    // サイドバー（単語辞書を使用）
    translations.insert(
        SIDEBAR_TURN_MANAGEMENT,
        format!("{}{}", W::TURN, W::MANAGEMENT),
    ); // 複数単語の組み合わせ
    translations.insert(
        SIDEBAR_ADD_NEW_TURN,
        format!("新しい{}を{}", W::TURN, W::ADD),
    );
    translations.insert(SIDEBAR_PLAYERS, format!("{}一覧", W::PLAYER));
    translations.insert(
        SIDEBAR_START_GAME_PROMPT,
        format!("{}を{}してください", W::GAME, W::START),
    );
    translations.insert(SIDEBAR_TURN_PREFIX, W::TURN.to_string());
    translations.insert(SIDEBAR_CURRENT_TURN_PREFIX, format!("▶ {}", W::TURN));

    // セットアップダイアログ（単語辞書を使用）
    translations.insert(SETUP_TITLE, format!("{}{}", W::GAME, W::SETTINGS));
    translations.insert(SETUP_AREA_SELECTION, format!("{}{}", W::AREA, W::SELECTION));
    translations.insert(
        SETUP_PLAYER_SETTINGS,
        format!("{}{}", W::PLAYER, W::SETTINGS),
    );
    translations.insert(SETUP_PLAYER_COUNT, format!("{}{}:", W::PLAYER, W::COUNT));
    translations.insert(
        SETUP_PLAYER_CONFIG,
        format!("各{}の{}:", W::PLAYER, W::CONFIG),
    );
    translations.insert(
        SETUP_CONFIG_NOTE,
        "（実装中: デフォルト設定で開始可能）".to_string(),
    );
    translations.insert(SETUP_START_GAME, format!("{}{}", W::GAME, W::START));
    translations.insert(SETUP_CANCEL, W::CANCEL.to_string());

    // メインコンテンツ（単語辞書を使用）
    translations.insert(MAIN_DRAWING_MODE, format!("{}{}:", W::DRAWING, W::MODE));
    translations.insert(MAIN_DRAWING_NONE, W::NONE.to_string());
    translations.insert(
        MAIN_DRAWING_CLICK_LINE,
        format!("{}で{}", W::CLICK, W::LINE),
    );
    translations.insert(MAIN_DRAWING_FREEHAND, W::FREEHAND.to_string());
    translations.insert(
        MAIN_DISCUSSION_INFO,
        format!("{}{}情報", W::DISCUSSION, W::TURN),
    );
    translations.insert(MAIN_KILLED_PLAYER, format!("{}{}:", W::KILLED, W::PLAYER));
    translations.insert(MAIN_KILL_LOCATION, format!("殺害{}（証言）:", W::LOCATION));
    translations.insert(MAIN_LOCATION_UNSET, W::UNSET.to_string());
    translations.insert(
        MAIN_SET_LOCATION_BTN,
        format!("マップ上で{}して{}", W::CLICK, W::SET),
    );
    translations.insert(
        MAIN_LOCATION_NOTE,
        "（実装中: マップ上で右クリックで設定予定）".to_string(),
    );
    translations.insert(MAIN_NOTES, format!("{}:", W::NOTES));
    translations.insert(MAIN_NO_SELECTION, W::NONE.to_string());
    translations.insert(MAIN_SELECT_PROMPT, "{}してください".to_string());
    translations.insert(MAIN_STATUS_ALIVE, W::ALIVE.to_string());
    translations.insert(MAIN_STATUS_DEAD, W::DEAD.to_string());
    translations.insert(MAIN_WELCOME_TITLE, "Among Us 補助ツール".to_string());
    translations.insert(
        MAIN_WELCOME_MESSAGE,
        format!("新しい{}を{}してください", W::GAME, W::START),
    );

    translations
}
