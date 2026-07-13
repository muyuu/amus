use std::collections::HashSet;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Color, Role, Sabotage};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerId(Uuid);

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PlayerState {
    Alive,
    Killed,
    Ejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub role: Role,
    pub color: Color,
    pub name: String,
    pub state: PlayerState,
    pub death: Option<usize>,        // 何ターン目か（1始まり）
    pub done_button: bool,           // ボタンを押したかどうか
    pub resolved: HashSet<Sabotage>, // 解決済みのサボタージュ種別
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
            resolved: HashSet::new(),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.state == PlayerState::Killed
    }

    pub fn is_ejected(&self) -> bool {
        self.state == PlayerState::Ejected
    }

    /// 指定サボタージュ種別を解決済みか。
    pub fn is_resolved(&self, kind: Sabotage) -> bool {
        self.resolved.contains(&kind)
    }
}
