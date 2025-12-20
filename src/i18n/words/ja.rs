// 日本語の基本単語辞書
pub struct JapaneseWords;

#[allow(dead_code)]
impl JapaneseWords {
    // アプリ全体
    pub const APP_TITLE: &'static str = "メモングアス";

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

    // アモアス用語
    pub const SABOTAGE: &'static str = "サボタージュ";
    pub const SABOTAGE_COMMS: &'static str = "通信妨害";
    pub const SABOTAGE_LIGHTS: &'static str = "停電";
    pub const SABOTAGE_O2: &'static str = "酸素妨害";
    pub const SABOTAGE_REACTOR: &'static str = "原子炉故障";
    pub const SABOTAGE_DOORS: &'static str = "ドア封鎖";

    pub const VENT: &'static str = "ベント";

    // The Skeld 部屋名
    pub const SKELD_ROOMS: &'static [&'static str] = &[
        "カフェテリア",
        "Cafeteria",
        "ウェポン",
        "Weapons",
        "ナビゲーション",
        "ナビ",
        "Navigation",
        "O2",
        "酸素",
        "シールド",
        "Shields",
        "コミュニケーション",
        "コムズ",
        "Communications",
        "ストレージ",
        "Storage",
        "アドミン",
        "Admin",
        "エレクトリカル",
        "エレキ",
        "Electrical",
        "ロワーエンジン",
        "Lower Engine",
        "リアクター",
        "Reactor",
        "セキュリティ",
        "Security",
        "アッパーエンジン",
        "Upper Engine",
        "メドベイ",
        "MedBay",
    ];

    // MIRA HQ 部屋名
    pub const MIRA_ROOMS: &'static [&'static str] = &[
        "ランチパッド",
        "Launchpad",
        "カフェテリア",
        "Cafeteria",
        "バルコニー",
        "Balcony",
        "リアクター",
        "Reactor",
        "ラボ",
        "ラボラトリー",
        "Laboratory",
        "オフィス",
        "Office",
        "アドミン",
        "Admin",
        "グリーンハウス",
        "Greenhouse",
        "メドベイ",
        "MedBay",
        "コミュニケーション",
        "コムズ",
        "Communications",
        "ロッカールーム",
        "Locker Room",
    ];

    // Polus 部屋名
    pub const POLUS_ROOMS: &'static [&'static str] = &[
        "オフィス",
        "Office",
        "アドミン",
        "Admin",
        "コミュニケーション",
        "コムズ",
        "Communications",
        "エレクトリカル",
        "エレキ",
        "Electrical",
        "O2",
        "酸素",
        "ウェポン",
        "Weapons",
        "ストレージ",
        "Storage",
        "ドリル",
        "ドロップシップ",
        "Dropship",
        "ラボ",
        "ラボラトリー",
        "Laboratory",
        "スペシメン",
        "Specimen Room",
        "メドベイ",
        "MedBay",
        "ボイラールーム",
        "Boiler Room",
        "セキュリティ",
        "Security",
    ];

    // AirShip 部屋名
    pub const AIRSHIP_ROOMS: &'static [&'static str] = &[
        "ミーティングルーム",
        "ミーティング",
        "金庫",
        "金庫室",
        "宿舎",
        "宿舎前通路",
        "昇降機",
        "梯子",
        "ヌーン",
        "アーカイブ",
        "ラウンジ",
        "トイレ",
        "コックピット",
        "アドミン",
        "通信室",
        "コミュ",
        "エンジンルーム",
        "エンジン",
        "メイン",
        "シャワー",
        "ロミジュリ",
        "貨物",
        "貨物室",
        "おぼえげ",
        "武器庫",
        "武器上",
        "武器下",
        "キッチン",
        "セキュ",
        "カメラ",
        "テープ",
        "電気室",
        "エレキ",
        "エレキカチ",
        "エレキぐるぐる",
        "エレキガチャガチャ",
        "診察室",
        "バイタル",
        "展望",
        "展望デッキ",
    ];
}
