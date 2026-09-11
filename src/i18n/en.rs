use super::keys::TextKey;
use super::words::en::EnglishWords as W;

/// キーを現在言語の文字列に変換する。
/// `TextKey` 全件を網羅し、コンパイル時に翻訳漏れを防ぐ。
pub fn translate(key: TextKey) -> String {
    use TextKey::*;
    match key {
        AppTitle => W::APP_TITLE.to_string(),
        MenuFile => W::FILE.to_string(),
        MenuNewGame => format!("{} {}", W::NEW, W::GAME),
        MenuLoadGame => format!("{} {}", W::LOAD, W::GAME),
        MenuSaveGame => format!("{} {}", W::SAVE, W::GAME),
        MenuLanguage => W::LANGUAGE.to_string(),
        SidebarTurnManagement => format!("{} {}", W::TURN, W::MANAGEMENT),
        SidebarAddNewTurn => format!("{} {} {}", W::ADD, W::NEW, W::TURN),
        SidebarPlayers => W::PLAYER.to_string(),
        SidebarStartGamePrompt => format!(
            "Please {} a new {}",
            W::START.to_lowercase(),
            W::GAME.to_lowercase()
        ),
        SidebarTurnPrefix => W::TURN.to_string(),
        SidebarCurrentTurnPrefix => format!("▶ {}", W::TURN),
        SetupTitle => format!("{} {}", W::GAME, W::SETTINGS),
        SetupAreaSelection => format!("{} {}", W::AREA, W::SELECTION),
        SetupPlayerSettings => format!("{} {}", W::PLAYER, W::SETTINGS),
        SetupPlayerCount => format!("{} {}:", W::PLAYER, W::COUNT),
        SetupPlayerConfig => format!("{} {}:", W::PLAYER, W::CONFIG),
        SetupConfigNote => "(In development: Default settings available)".to_string(),
        SetupPlayerName => W::NAME.to_string(),
        SetupPlayerColor => W::COLOR.to_string(),
        SetupStartGame => format!("{} {}", W::START, W::GAME),
        SetupCancel => W::CANCEL.to_string(),
        MainDrawingMode => format!("{} {}:", W::DRAWING, W::MODE),
        MainDrawingNone => W::NONE.to_string(),
        MainDrawingClickLine => format!("{} to {}", W::CLICK, W::LINE),
        MainDrawingFreehand => W::FREEHAND.to_string(),
        MainDiscussionInfo => format!("{} {} {}", W::DISCUSSION, W::TURN, W::INFORMATION),
        MainKilledPlayer => format!("{} {}:", W::KILLED, W::PLAYER),
        MainKillLocation => format!("Kill {} (Testimony):", W::LOCATION),
        MainLocationUnset => W::UNSET.to_string(),
        MainSetLocationBtn => format!("{} on map to {}", W::CLICK, W::SET.to_lowercase()),
        MainLocationNote => "(In development: Right-click on map planned)".to_string(),
        MainNotes => format!("{}:", W::NOTES),
        MainNoSelection => W::NONE.to_string(),
        MainSelectPrompt => "Please select".to_string(),
        MainStatusAlive => W::ALIVE.to_string(),
        MainStatusDead => W::DEAD.to_string(),
        MainWelcomeTitle => W::APP_TITLE.to_string(),
        MainWelcomeMessage => format!(
            "Please {} a new {}",
            W::START.to_lowercase(),
            W::GAME.to_lowercase()
        ),
        PlayerInfoButtonDone => "Done".to_string(),
        PlayerInfoButtonNotDone => "Meeting".to_string(),
        EraserButton => W::ERASER.to_string(),
        Sabotage => W::SABOTAGE.to_string(),
        SabotageComms => W::SABOTAGE_COMMS.to_string(),
        SabotageLights => W::SABOTAGE_LIGHTS.to_string(),
        SabotageReactor => W::SABOTAGE_REACTOR.to_string(),
        SabotageO2 => W::SABOTAGE_O2.to_string(),
        SabotageDoors => W::SABOTAGE_DOORS.to_string(),
        VoiceMemoTitle => W::VOICE_MEMO.to_string(),
        VoiceMemoModelDownloading => {
            format!("⏳ {} {}...", W::DOWNLOADING, W::MODEL.to_lowercase())
        }
        VoiceMemoModelNotFound => format!("⚠ {} not found", W::WHISPER_MODEL),
        VoiceMemoModelRequired => format!(
            "{} requires {} {}",
            W::VOICE_RECOGNITION,
            W::MODEL_REQUIRED_SIZE,
            W::MODEL.to_lowercase()
        ),
        VoiceMemoDownloadModel => format!("📥 {} {}", W::DOWNLOAD, W::MODEL.to_lowercase()),
        VoiceMemoMicUnavailable => format!("⚠ {} unavailable", W::MIC),
        VoiceMemoEnd => W::END.to_string(),
        VoiceMemoTranscribing => format!("⏳ {}...", W::TRANSCRIBING),
        VoiceMemoStartTurn => format!("▶ {} {}", W::START, W::TURN),
        VoiceMemoClear => format!("🗑 {}", W::CLEAR),
        VoiceMemoError => W::ERROR.to_string(),
        VoiceMemoStartPrompt => format!(
            "Please {} a {}",
            W::START.to_lowercase(),
            W::TURN.to_lowercase()
        ),
        VoiceMemoRecordHint => W::RECORD_BUTTON_HINT.to_string(),
        VoiceMemoMemoCount => W::NOTES.to_string(),
        VoiceMemoTurnTime => format!("{} {}", W::TURN, W::TIME),
    }
}
