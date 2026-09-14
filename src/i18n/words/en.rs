// 英語の基本単語辞書
pub struct EnglishWords;

#[allow(dead_code)]
impl EnglishWords {
    // アプリ全体
    pub const APP_TITLE: &'static str = "Memong us";

    // 基本動詞
    pub const START: &'static str = "Start";
    pub const END: &'static str = "End";
    pub const STOP: &'static str = "Stop";
    pub const CANCEL: &'static str = "Cancel";
    pub const ADD: &'static str = "Add";
    pub const SAVE: &'static str = "Save";
    pub const LOAD: &'static str = "Load";
    pub const NEW: &'static str = "New";
    pub const SET: &'static str = "Set";
    pub const CLEAR: &'static str = "Clear";
    pub const RECORD: &'static str = "Record";
    pub const DOWNLOAD: &'static str = "Download";
    pub const RESET: &'static str = "Reset";
    pub const CLOSE: &'static str = "Close";

    // 基本名詞
    pub const GAME: &'static str = "Game";
    pub const TURN: &'static str = "Turn";
    pub const PLAYER: &'static str = "Player";
    pub const MANAGEMENT: &'static str = "Management";
    pub const FILE: &'static str = "File";
    pub const LANGUAGE: &'static str = "Language";
    pub const AREA: &'static str = "Area";
    pub const LOCATION: &'static str = "Location";
    pub const NOTES: &'static str = "Notes";
    pub const SELECTION: &'static str = "Selection";
    pub const SETTINGS: &'static str = "Settings";
    pub const DISPLAY: &'static str = "Display";
    pub const VOICE: &'static str = "Voice";
    pub const APP: &'static str = "App";
    pub const WIDTH: &'static str = "Width";
    pub const VERSION: &'static str = "Version";
    pub const CONFIG: &'static str = "Configuration";
    pub const COUNT: &'static str = "Count";
    pub const MODE: &'static str = "Mode";
    pub const DRAWING: &'static str = "Drawing";
    pub const DISCUSSION: &'static str = "Discussion";
    pub const INFORMATION: &'static str = "Information";
    pub const NAME: &'static str = "Name";
    pub const COLOR: &'static str = "Color";
    pub const ERASER: &'static str = "Eraser";

    // 状態・形容詞
    pub const ALIVE: &'static str = "Alive";
    pub const DEAD: &'static str = "Dead";
    pub const KILLED: &'static str = "Killed";
    pub const NONE: &'static str = "None";
    pub const UNSET: &'static str = "Unset";

    // 操作関連
    pub const CLICK: &'static str = "Click";
    pub const LINE: &'static str = "Line";
    pub const FREEHAND: &'static str = "Freehand";

    // アモアス用語
    pub const SABOTAGE: &'static str = "Sabotage";
    pub const SABOTAGE_COMMS: &'static str = "Comms Sabotage";
    pub const SABOTAGE_LIGHTS: &'static str = "Lights Sabotage";
    pub const SABOTAGE_REACTOR: &'static str = "Meltdown Reactor";
    pub const SABOTAGE_O2: &'static str = "O2 Depletion";
    pub const SABOTAGE_DOORS: &'static str = "Doors Sabotage";

    pub const SPAWN: &'static str = "Spawn";
    pub const VENT: &'static str = "Vent";
    pub const TASK: &'static str = "Task";

    // 追加の名詞
    pub const MODEL: &'static str = "Model";
    pub const MIC: &'static str = "Microphone";
    pub const ERROR: &'static str = "Error";
    pub const TIME: &'static str = "Time";
    pub const SECOND: &'static str = "s";
    pub const ITEM: &'static str = "items";

    // 音声メモ関連
    pub const VOICE_MEMO: &'static str = "Voice Memo";
    pub const TRANSCRIBING: &'static str = "Transcribing";
    pub const DOWNLOADING: &'static str = "Downloading";
    pub const VOICE_RECOGNITION: &'static str = "Voice recognition";
    pub const WHISPER_MODEL: &'static str = "Whisper model";
    pub const RECORD_BUTTON_HINT: &'static str = "Press record to capture speech";
}
