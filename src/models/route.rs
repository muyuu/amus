use serde::{Deserialize, Serialize};

use super::Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub user_id: usize,     // Userのインデックス参照
    pub points: Vec<Point>, // 軌跡のポイント（開始地点はspawn_locationsから取得）
}

impl Route {
    pub fn new(user_id: usize) -> Self {
        Self {
            user_id,
            points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }
}
