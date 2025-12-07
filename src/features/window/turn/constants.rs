use egui::*;

pub struct TurnConstants;

impl TurnConstants {
    pub const TURN_PANEL_DEFAULT_POS_OFFSET_X: f32 = 10.0;
    pub const TURN_PANEL_DEFAULT_POS_OFFSET_Y: f32 = 10.0;

    pub const TURN_GRID_SPACE: f32 = 4.0;
    pub const TURN_GRID_ROWS: usize = 4;

    pub const TURN_BUTTON_FONT_SIZE: f32 = 18.0;
    pub const TURN_BUTTON_BG_COLOR: Color32 = Color32::from_rgb(50, 50, 50);
    pub const TURN_BUTTON_BG_COLOR_HIGHLIGHT: Color32 = Color32::RED;
}
