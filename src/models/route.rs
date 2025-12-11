use std::vec;

use serde::{Deserialize, Serialize};

use crate::models::player::PlayerId;

use super::Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Route {
    Erase(Erase),
    Draw(Draw),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draw {
    pub player_id: PlayerId,    // Playerのインデックス参照
    pub lines: Vec<Vec<Point>>, // 軌跡のポイント（開始地点はspawn_locationsから取得）
}

impl Route {
    pub(crate) fn lines_mut(&mut self) -> &mut Vec<Vec<Point>> {
        match self {
            Route::Draw(draw) => &mut draw.lines,
            Route::Erase(erase) => &mut erase.lines,
        }
    }
}

impl Draw {
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            player_id,
            lines: vec![vec![]],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erase {
    pub lines: Vec<Vec<Point>>,
}

impl Erase {
    pub fn new() -> Self {
        Self {
            lines: vec![vec![]],
        }
    }
}

impl Default for Erase {
    fn default() -> Self {
        Erase::new()
    }
}
