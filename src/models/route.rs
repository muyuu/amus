use std::vec;

use serde::{Deserialize, Serialize};

use super::Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Route {
    Erase(Erase),
    Draw(Draw),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draw {
    pub user_id: usize,         // Userのインデックス参照
    pub lines: Vec<Vec<Point>>, // 軌跡のポイント（開始地点はspawn_locationsから取得）
}

impl Route {
    pub fn add_point(&mut self, point: Point) {
        let lines = self.lines_mut();
        let last_line = match lines.last_mut() {
            Some(line) => line,
            None => return,
        };

        // 直前のポイントと同じ場合は追加しない
        let last_point = last_line.last();
        if let Some(lp) = last_point {
            if lp.x == point.x && lp.y == point.y {
                return;
            }
        }
        last_line.push(point);
    }

    fn lines_mut(&mut self) -> &mut Vec<Vec<Point>> {
        match self {
            Route::Draw(draw) => &mut draw.lines,
            Route::Erase(erase) => &mut erase.lines,
        }
    }
}

impl Draw {
    pub fn new(user_id: usize) -> Self {
        Self {
            user_id,
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
