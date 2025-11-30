use egui::Color32;

pub struct AppConstants;

#[allow(dead_code)]
impl AppConstants {
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;

    /// ウィンドウ背景色
    pub const WINDOW_BG_COLOR_DARK: Color32 = Color32::from_rgb(0, 0, 0);
    pub const WINDOW_BG_COLOR_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);

    /// コンポーネントの定数たち
    pub const COM_BG_LABEL_PADDING_X: f32 = 8.0;
    pub const COM_BG_LABEL_PADDING_Y: f32 = 4.0;

    /// ユーザー情報表示の定数たち
    pub const USER_INFO_COLOR_BOX_SIZE: f32 = 20.0;
}
