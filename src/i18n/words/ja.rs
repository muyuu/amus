// 日本語の基本単語辞書
pub struct JapaneseWords;

impl JapaneseWords {
    // 基本動詞
    pub const START: &'static str = "開始";
    pub const CANCEL: &'static str = "キャンセル";
    pub const ADD: &'static str = "追加";
    pub const SAVE: &'static str = "保存";
    pub const LOAD: &'static str = "読み込み";
    pub const NEW: &'static str = "新規";
    pub const SET: &'static str = "設定";

    // 基本名詞
    pub const GAME: &'static str = "ゲーム";
    pub const TURN: &'static str = "ターン";
    pub const PLAYER: &'static str = "プレイヤー";
    pub const MANAGEMENT: &'static str = "管理";
    pub const FILE: &'static str = "ファイル";
    pub const LANGUAGE: &'static str = "言語";
    pub const AREA: &'static str = "エリア";
    pub const LOCATION: &'static str = "場所";
    pub const NOTES: &'static str = "メモ";
    pub const SELECTION: &'static str = "選択";
    pub const SETTINGS: &'static str = "設定";
    pub const CONFIG: &'static str = "設定";
    pub const COUNT: &'static str = "人数";
    pub const MODE: &'static str = "モード";
    pub const DRAWING: &'static str = "描画";
    pub const DISCUSSION: &'static str = "議論";
    pub const NAME: &'static str = "名前";
    pub const COLOR: &'static str = "色";
    pub const ERASER: &'static str = "消しゴム";

    // 状態・形容詞
    pub const ALIVE: &'static str = "生存";
    pub const DEAD: &'static str = "死亡";
    pub const KILLED: &'static str = "殺害された";
    pub const NONE: &'static str = "なし";
    pub const UNSET: &'static str = "未設定";

    // 操作関連
    pub const CLICK: &'static str = "クリック";
    pub const LINE: &'static str = "線";
    pub const FREEHAND: &'static str = "フリーハンド";
}
