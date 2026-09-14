/// ストレージで使用するキーの定数定義
pub struct StorageKeys;

impl StorageKeys {
    /// セットアップ状態
    pub const SETUP_STATE: &'static str = "setup_state";

    /// ゲーム状態
    pub const GAME: &'static str = "game";

    /// 現在のウェーブインデックス
    pub const CURRENT_WAVE_INDEX: &'static str = "current_wave_index";

    /// デバッグビューの表示状態
    pub const SHOW_DEBUG_VIEW: &'static str = "show_debug_view";

    /// 消しゴムモードの状態
    pub const ERASE_MODE: &'static str = "erase_mode";

    /// UI 拡大率（None = ネイティブ DPI 追従）
    pub const UI_SCALE: &'static str = "ui_scale";

    /// 軌跡描画の線の太さ
    pub const ROUTE_LINE_WIDTH: &'static str = "route_line_width";
}
