use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Crew,
    Imposter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Color {
    Red = 0,
    Blue = 1,
    Green = 2,
    Pink = 3,
    Orange = 4,
    Yellow = 5,
    Black = 6,
    White = 7,
    Purple = 8,
    Brown = 9,
    Cyan = 10,
    Lime = 11,
    Maroon = 12,
    Rose = 13,
    Banana = 14,
    Gray = 15,
    Tan = 16,
    Coral = 17,
}

impl Color {
    pub fn to_rgb(&self) -> [u8; 3] {
        match self {
            Color::Red => [215, 30, 34],
            Color::Blue => [29, 60, 233],
            Color::Green => [27, 145, 62],
            Color::Pink => [255, 99, 212],
            Color::Orange => [255, 141, 28],
            Color::Yellow => [255, 255, 103],
            Color::Black => [74, 86, 94],
            Color::White => [233, 247, 255],
            Color::Purple => [120, 61, 210],
            Color::Brown => [128, 88, 45],
            Color::Cyan => [68, 255, 247],
            Color::Lime => [91, 254, 75],
            Color::Maroon => [108, 43, 61],
            Color::Rose => [255, 214, 236],
            Color::Banana => [255, 255, 190],
            Color::Gray => [131, 151, 167],
            Color::Tan => [159, 153, 137],
            Color::Coral => [236, 117, 120],
        }
    }

    pub fn to_egui_color(&self) -> egui::Color32 {
        let rgb = self.to_rgb();
        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    pub fn all() -> Vec<Color> {
        vec![
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
            Color::Gray,
            Color::Tan,
            Color::Coral,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Color::Red => "Red",
            Color::Blue => "Blue",
            Color::Green => "Green",
            Color::Pink => "Pink",
            Color::Orange => "Orange",
            Color::Yellow => "Yellow",
            Color::Black => "Black",
            Color::White => "White",
            Color::Purple => "Purple",
            Color::Brown => "Brown",
            Color::Cyan => "Cyan",
            Color::Lime => "Lime",
            Color::Maroon => "Maroon",
            Color::Rose => "Rose",
            Color::Banana => "Banana",
            Color::Gray => "Gray",
            Color::Tan => "Tan",
            Color::Coral => "Coral",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub name: String,
    pub id: String,
}

impl Area {
    pub fn new(name: String, id: String) -> Self {
        Self { name, id }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    pub routes: Vec<Route>,
    pub killed: Option<usize>,        // 殺害されたユーザーのID
    pub kill_location: Option<Point>, // 殺害場所（証言ベース）
    pub notes: String,                // 議論ターンでのメモ
    pub spawn_locations: std::collections::HashMap<usize, Point>, // ユーザーID -> 出現場所
    pub end_locations: std::collections::HashMap<usize, Point>, // ユーザーID -> 終了時位置
}

impl Wave {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            killed: None,
            kill_location: None,
            notes: String::new(),
            spawn_locations: std::collections::HashMap::new(),
            end_locations: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub waves: Vec<Wave>,
    pub area: Area,
    pub users: Vec<User>,
}

impl Game {
    pub fn new(area: Area, users: Vec<User>) -> Self {
        Self {
            waves: Vec::new(),
            area,
            users,
        }
    }

    pub fn add_wave(&mut self) {
        self.waves.push(Wave::new());
    }

    pub fn get_wave(&self, index: usize) -> Option<&Wave> {
        self.waves.get(index)
    }

    pub fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }
}
