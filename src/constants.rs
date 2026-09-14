use egui::*;

pub struct AppConstants;

impl AppConstants {
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;

    /// UI 拡大率（pixels_per_point）のユーザー指定可能な範囲。
    /// 範囲外の値は設定時にクランプされる。
    pub const UI_SCALE_MIN: f32 = 0.5;
    pub const UI_SCALE_MAX: f32 = 3.0;

    /// 軌跡描画の線の太さのユーザー指定可能な範囲。
    pub const ROUTE_LINE_WIDTH_MIN: f32 = 2.0;
    pub const ROUTE_LINE_WIDTH_MAX: f32 = 16.0;
    pub const ROUTE_LINE_WIDTH_DEFAULT: f32 = 6.0;

    /// アプリの地の色。
    ///
    /// マップ画像は透過なので、画像の背景・画像の外側・パネル枠のすべてがこの色になる。
    /// テーマを切り替えるときはここだけを差し替える。
    pub const BG_COLOR: Color32 = Color32::from_rgb(0, 0, 0);

    /// コンポーネントの定数たち
    pub const COM_BG_LABEL_PADDING_X: f32 = 4.0;
    pub const COM_BG_LABEL_PADDING_Y: f32 = 4.0;
}

pub struct PanelIds;
impl PanelIds {
    pub const PLAYER_INFO: &str = "player_info_panel";
}

pub struct GridIds;
impl GridIds {
    pub const COMMS: &str = "comms_grid";
    pub const LIGHTS: &str = "lights_grid";
    pub const O2: &str = "o2_grid";
    pub const REACTOR: &str = "reactor_grid";
    pub const TURN: &str = "turn_grid";
    pub const DEBUG_TABLE: &str = "debug_table_grid";
}

pub struct SelectIds;
impl SelectIds {
    pub const SETUP_AREA: &str = "setup_area_select";
    pub const SETUP_PLAYER_COLOR: &str = "setup_player_color_select";
    pub const SETTINGS_UI_SCALE: &str = "settings_ui_scale_select";
    /// 音声タブの入力デバイス選択（ネイティブの voice_memo 機能専用）。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub const SETTINGS_INPUT_DEVICE: &str = "settings_input_device_select";
}

pub struct ContextIds;
impl ContextIds {
    pub const ASSET_MANAGER: &str = "asset_manager";
}
