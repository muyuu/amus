use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{Point, Route};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    pub routes: Vec<Route>,
    pub killed: Option<usize>,                  // 殺害されたユーザーのID
    pub kill_location: Option<Point>,           // 殺害場所（証言ベース）
    pub notes: String,                          // 議論ターンでのメモ
    pub spawn_locations: HashMap<usize, Point>, // ユーザーID -> 出現場所
    pub end_locations: HashMap<usize, Point>,   // ユーザーID -> 終了時位置
}

impl Wave {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            killed: None,
            kill_location: None,
            notes: String::new(),
            spawn_locations: HashMap::new(),
            end_locations: HashMap::new(),
        }
    }
}
