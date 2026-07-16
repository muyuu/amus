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

/// 1 ゲーム中のプレイヤーの進行状態。永続属性（id/role/color/name）と分離し、
/// 記録項目が増えても Player トップレベルを膨張させない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgress {
    pub state: PlayerState,
    pub death: Option<usize>,        // 何ターン目か（1始まり）
    pub done_button: bool,           // ボタンを押したかどうか
    pub resolved: HashSet<Sabotage>, // 解決済みのサボタージュ種別
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            state: PlayerState::Alive,
            death: None,
            done_button: false,
            resolved: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub role: Role,
    pub color: Color,
    pub name: String,
    pub progress: PlayerProgress,
}

impl Player {
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self {
            id: PlayerId(Uuid::new_v4()),
            role,
            color,
            name,
            progress: PlayerProgress::default(),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.progress.state == PlayerState::Killed
    }

    pub fn is_ejected(&self) -> bool {
        self.progress.state == PlayerState::Ejected
    }

    /// 指定サボタージュ種別を解決済みか。
    pub fn is_resolved(&self, kind: Sabotage) -> bool {
        self.progress.resolved.contains(&kind)
    }
}
