use serde::{Deserialize, Serialize};

use super::Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub user_id: usize,         // Userのインデックス参照
    pub lines: Vec<Vec<Point>>, // 軌跡のポイント（開始地点はspawn_locationsから取得）
}

impl Route {
    pub fn new(user_id: usize) -> Self {
        Self {
            user_id,
            lines: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: Vec<Point>) {
        self.lines.push(line);
    }

    pub fn add_point(&mut self, point: Point) {
        let last_line = match self.lines.last_mut() {
            Some(line) => line,
            None => return,
        };

        last_line.push(point);
    }
}
