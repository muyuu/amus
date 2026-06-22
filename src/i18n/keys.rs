//! 翻訳キー。
//!
//! `TextKey` の列挙子は宣言順の連番（`as usize` で 0..COUNT）で、翻訳テーブル配列の
//! インデックスとして使う（`super::Translator` 参照）。呼び出し側は従来どおり
//! `keys::APP_TITLE` 等の定数名で参照する（型は `&str` から `TextKey` に変わった）。

/// キー一覧を 1 箇所で宣言し、enum・全件配列・定数名を同時に生成する。
/// `定数名 => 列挙子名` を 1 行ずつ並べる。
macro_rules! text_keys {
    ($($const_name:ident => $variant:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TextKey {
            $($variant),+
        }

        impl TextKey {
            /// 全キーを宣言順（= `as usize` の順）に並べた配列。テーブル構築に使う。
            pub const ALL: &'static [TextKey] = &[$(TextKey::$variant),+];
        }

        // キーの全集合を網羅する catalog。表示側に未配線のキーも含むため
        // 個々の定数は未使用でも許容する。
        $(#[allow(dead_code)] pub const $const_name: TextKey = TextKey::$variant;)+
    };
}

text_keys! {
    APP_TITLE => AppTitle,
    MENU_FILE => MenuFile,
    MENU_NEW_GAME => MenuNewGame,
    MENU_LOAD_GAME => MenuLoadGame,
    MENU_SAVE_GAME => MenuSaveGame,
    MENU_LANGUAGE => MenuLanguage,
    SIDEBAR_TURN_MANAGEMENT => SidebarTurnManagement,
    SIDEBAR_ADD_NEW_TURN => SidebarAddNewTurn,
    SIDEBAR_PLAYERS => SidebarPlayers,
    SIDEBAR_START_GAME_PROMPT => SidebarStartGamePrompt,
    SIDEBAR_TURN_PREFIX => SidebarTurnPrefix,
    SIDEBAR_CURRENT_TURN_PREFIX => SidebarCurrentTurnPrefix,
    SETUP_TITLE => SetupTitle,
    SETUP_AREA_SELECTION => SetupAreaSelection,
    SETUP_PLAYER_SETTINGS => SetupPlayerSettings,
    SETUP_PLAYER_COUNT => SetupPlayerCount,
    SETUP_PLAYER_CONFIG => SetupPlayerConfig,
    SETUP_CONFIG_NOTE => SetupConfigNote,
    SETUP_PLAYER_NAME => SetupPlayerName,
    SETUP_PLAYER_COLOR => SetupPlayerColor,
    SETUP_START_GAME => SetupStartGame,
    SETUP_CANCEL => SetupCancel,
    MAIN_DRAWING_MODE => MainDrawingMode,
    MAIN_DRAWING_NONE => MainDrawingNone,
    MAIN_DRAWING_CLICK_LINE => MainDrawingClickLine,
    MAIN_DRAWING_FREEHAND => MainDrawingFreehand,
    MAIN_DISCUSSION_INFO => MainDiscussionInfo,
    MAIN_KILLED_PLAYER => MainKilledPlayer,
    MAIN_KILL_LOCATION => MainKillLocation,
    MAIN_LOCATION_UNSET => MainLocationUnset,
    MAIN_SET_LOCATION_BTN => MainSetLocationBtn,
    MAIN_LOCATION_NOTE => MainLocationNote,
    MAIN_NOTES => MainNotes,
    MAIN_NO_SELECTION => MainNoSelection,
    MAIN_SELECT_PROMPT => MainSelectPrompt,
    MAIN_STATUS_ALIVE => MainStatusAlive,
    MAIN_STATUS_DEAD => MainStatusDead,
    MAIN_WELCOME_TITLE => MainWelcomeTitle,
    MAIN_WELCOME_MESSAGE => MainWelcomeMessage,
    PLAYER_INFO_BUTTON_DONE => PlayerInfoButtonDone,
    PLAYER_INFO_BUTTON_NOT_DONE => PlayerInfoButtonNotDone,
    ERASER_BUTTON => EraserButton,
    SABOTAGE => Sabotage,
    SABOTAGE_COMMS => SabotageComms,
    SABOTAGE_LIGHTS => SabotageLights,
    SABOTAGE_REACTOR => SabotageReactor,
    SABOTAGE_O2 => SabotageO2,
    SABOTAGE_DOORS => SabotageDoors,
    VOICE_MEMO_TITLE => VoiceMemoTitle,
    VOICE_MEMO_MODEL_DOWNLOADING => VoiceMemoModelDownloading,
    VOICE_MEMO_MODEL_NOT_FOUND => VoiceMemoModelNotFound,
    VOICE_MEMO_MODEL_REQUIRED => VoiceMemoModelRequired,
    VOICE_MEMO_DOWNLOAD_MODEL => VoiceMemoDownloadModel,
    VOICE_MEMO_MIC_UNAVAILABLE => VoiceMemoMicUnavailable,
    VOICE_MEMO_END => VoiceMemoEnd,
    VOICE_MEMO_REC => VoiceMemoRec,
    VOICE_MEMO_STOP => VoiceMemoStop,
    VOICE_MEMO_TRANSCRIBING => VoiceMemoTranscribing,
    VOICE_MEMO_RECORD => VoiceMemoRecord,
    VOICE_MEMO_START_TURN => VoiceMemoStartTurn,
    VOICE_MEMO_CLEAR => VoiceMemoClear,
    VOICE_MEMO_ERROR => VoiceMemoError,
    VOICE_MEMO_START_PROMPT => VoiceMemoStartPrompt,
    VOICE_MEMO_RECORD_HINT => VoiceMemoRecordHint,
    VOICE_MEMO_MEMO_COUNT => VoiceMemoMemoCount,
    VOICE_MEMO_TURN_TIME => VoiceMemoTurnTime,
}
