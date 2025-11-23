use crate::models::{Area, Color, Role};

// デフォルトのプレイヤー人数定数
const DEFAULT_PLAYER_COUNT: usize = 8;

#[derive(Debug, Clone)]
pub struct PlayerSetup {
    pub name: String,
    pub color: Color,
    #[allow(dead_code)]
    pub role: Option<Role>, // セットアップ時は未決定
}

impl PlayerSetup {
    pub fn new(index: usize, color: Color) -> Self {
        Self {
            name: format!("Player{}", index + 1),
            color,
            role: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetupState {
    pub selected_area: Area,
    pub player_count: usize,
    pub players: Vec<PlayerSetup>,
}

impl Default for SetupState {
    fn default() -> Self {
        let default_colors = [
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Pink,
            Color::Orange,
            Color::Yellow,
            Color::Black,
            Color::White,
            Color::Purple,
            Color::Brown,
            Color::Cyan,
            Color::Lime,
            Color::Maroon,
            Color::Rose,
            Color::Banana,
        ];

        let mut players = Vec::new();
        for i in 0..DEFAULT_PLAYER_COUNT {
            // デフォルトの人数分
            let color = default_colors
                .get(i % default_colors.len())
                .cloned()
                .unwrap_or(Color::Red);
            players.push(PlayerSetup::new(i, color));
        }

        Self {
            selected_area: Area::AirShip,
            player_count: DEFAULT_PLAYER_COUNT,
            players,
        }
    }
}
