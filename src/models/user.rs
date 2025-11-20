use serde::{Deserialize, Serialize};

use super::{Color, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub role: Role,
    pub color: Color,
    pub name: String,
    pub alive: bool,
    pub death: Option<usize>, // 何ターン目か（1始まり）
}

impl User {
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self {
            role,
            color,
            name,
            alive: true,
            death: None,
        }
    }
}
