use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Color, Role};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerId(Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PlayerState {
    Alive,
    Killed,
    Ejected,
}

impl PlayerState {
    pub fn next(&self) -> Self {
        match self {
            PlayerState::Alive => PlayerState::Killed,
            PlayerState::Killed => PlayerState::Ejected,
            PlayerState::Ejected => PlayerState::Alive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub role: Role,
    pub color: Color,
    pub name: String,
    pub state: PlayerState,
    pub death: Option<usize>, // 何ターン目か（1始まり）
    pub done_button: bool,    // ボタンを押したかどうか
}

impl Player {
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self {
            id: PlayerId(Uuid::new_v4()),
            role,
            color,
            name,
            state: PlayerState::Alive,
            death: None,
            done_button: false,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.state == PlayerState::Killed
    }

    pub fn is_ejected(&self) -> bool {
        self.state == PlayerState::Ejected
    }
}
