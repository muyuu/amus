use crate::models::{Area, Color, Role, Player};

// デフォルトのプレイヤー人数定数
const DEFAULT_PLAYER_COUNT: usize = 8;

#[derive(Debug, Clone)]
pub struct SetupState {
    pub selected_area: Area,
    pub player_count: usize,
    pub players: Vec<Player>,
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
            players.push(Player::new(Role::Crew, color, format!("Player {}", i + 1)));
        }

        Self {
            selected_area: Area::AirShip,
            player_count: DEFAULT_PLAYER_COUNT,
            players,
        }
    }
}
