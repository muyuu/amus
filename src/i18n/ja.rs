use super::keys::*;
use super::words::ja::JapaneseWords as W;
use std::collections::HashMap;

pub fn get_translations() -> HashMap<&'static str, String> {
    let mut translations = HashMap::new();

    // アプリケーション
    translations.insert(APP_TITLE, format!("{}", W::APP_TITLE));

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
    translations.insert(SETUP_AREA_SELECTION, W::AREA.to_string());
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
    translations.insert(SETUP_PLAYER_NAME, W::NAME.to_string());
    translations.insert(SETUP_PLAYER_COLOR, W::COLOR.to_string());
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
    translations.insert(MAIN_WELCOME_TITLE, format!("{}", W::APP_TITLE));
    translations.insert(
        MAIN_WELCOME_MESSAGE,
        format!("新しい{}を{}してください", W::GAME, W::START),
    );

    translations.insert(PLAYER_INFO_BUTTON_DONE, "済み".to_string());
    translations.insert(PLAYER_INFO_BUTTON_NOT_DONE, "会議".to_string());

    // 消しゴム
    translations.insert(ERASER_BUTTON, W::ERASER.to_string());

    // サボタージュ
    translations.insert(SABOTAGE, W::SABOTAGE.to_string());
    translations.insert(SABOTAGE_COMMS, W::SABOTAGE_COMMS.to_string());
    translations.insert(SABOTAGE_LIGHTS, W::SABOTAGE_LIGHTS.to_string());
    translations.insert(SABOTAGE_REACTOR, W::SABOTAGE_REACTOR.to_string());
    translations.insert(SABOTAGE_O2, W::SABOTAGE_O2.to_string());
    translations.insert(SABOTAGE_DOORS, W::SABOTAGE_DOORS.to_string());

    // 音声メモ
    translations.insert(VOICE_MEMO_TITLE, W::VOICE_MEMO.to_string());
    translations.insert(
        VOICE_MEMO_MODEL_DOWNLOADING,
        format!("⏳ {}を{}...", W::MODEL, W::DOWNLOADING),
    );
    translations.insert(
        VOICE_MEMO_MODEL_NOT_FOUND,
        format!("⚠ {}が見つかりません", W::WHISPER_MODEL),
    );
    translations.insert(
        VOICE_MEMO_MODEL_REQUIRED,
        format!(
            "{}には{}の{}が必要です",
            W::VOICE_RECOGNITION,
            W::MODEL_REQUIRED_SIZE,
            W::MODEL
        ),
    );
    translations.insert(
        VOICE_MEMO_DOWNLOAD_MODEL,
        format!("📥 {}を{}", W::MODEL, W::DOWNLOAD),
    );
    translations.insert(
        VOICE_MEMO_MIC_UNAVAILABLE,
        format!("⚠ {}が利用できません", W::MIC),
    );
    translations.insert(VOICE_MEMO_END, W::END.to_string());
    translations.insert(VOICE_MEMO_REC, "● REC".to_string());
    translations.insert(VOICE_MEMO_STOP, format!("⏹ {}", W::STOP));
    translations.insert(
        VOICE_MEMO_TRANSCRIBING,
        format!("⏳ {}...", W::TRANSCRIBING),
    );
    translations.insert(VOICE_MEMO_RECORD, format!("🎤 {}", W::RECORD));
    translations.insert(
        VOICE_MEMO_START_TURN,
        format!("▶ {}{}", W::TURN, W::START),
    );
    translations.insert(VOICE_MEMO_CLEAR, format!("🗑 {}", W::CLEAR));
    translations.insert(VOICE_MEMO_ERROR, W::ERROR.to_string());
    translations.insert(
        VOICE_MEMO_START_PROMPT,
        format!("{}を{}してください", W::TURN, W::START),
    );
    translations.insert(VOICE_MEMO_RECORD_HINT, W::RECORD_BUTTON_HINT.to_string());
    translations.insert(VOICE_MEMO_MEMO_COUNT, W::NOTES.to_string());
    translations.insert(VOICE_MEMO_TURN_TIME, format!("{}{}", W::TURN, W::TIME));

    translations
}
