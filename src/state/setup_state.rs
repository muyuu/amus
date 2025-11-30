use crate::models::{Area, Color, Player, Role};

// デフォルトのプレイヤー人数定数
const DEFAULT_PLAYER_COUNT: usize = 12;

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
            players.push(Player::new(Role::Crew, color, Self::get_default_player_name(i)));
        }

        Self {
            selected_area: Area::AirShip,
            player_count: DEFAULT_PLAYER_COUNT,
            players,
        }
    }
}

impl SetupState {
    fn get_default_player_name(i: usize) -> String {
        let default_names = Self::get_default_player_names();
        if i < default_names.len() {
            default_names[i].clone()
        } else {
            format!("Player{}", i + 1)
        }
    }

    fn get_default_player_names() -> Vec<String> {
        vec![
            "えんがわ".to_string(),
            "りょーちゃん".to_string(),
            "なあこ".to_string(),
            "あさ".to_string(),
            "ツバサ".to_string(),
            "にゃんばる".to_string(),
            "たぬころ".to_string(),
            "かめなし".to_string(),
            "レーツェル".to_string(),
            "何".to_string(),
            "檸檬".to_string(),
            "くりぼっくり".to_string(),
            "ファーム".to_string(),
        ]
    }
}

