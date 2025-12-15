use egui::*;

pub struct AppConstants;

impl AppConstants {
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;

    /// ウィンドウ
    pub const WINDOW_BG_COLOR_DARK: Color32 = Color32::from_rgb(0, 0, 0);
    #[allow(dead_code)]
    pub const WINDOW_BG_COLOR_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);

    /// コンポーネントの定数たち
    pub const COM_BG_LABEL_PADDING_X: f32 = 4.0;
    pub const COM_BG_LABEL_PADDING_Y: f32 = 4.0;
}

pub struct PanelIds;
impl PanelIds {
    pub const PLAYER_INFO: &str = "player_info_panel";
    pub const MENU: &str = "menu_panel";
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
}

pub struct ContextIds;
impl ContextIds {
    pub const ASSET_MANAGER: &str = "asset_manager";
}
