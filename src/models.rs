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
            Color::Red => [255, 0, 0],
            Color::Blue => [0, 0, 255],
            Color::Green => [0, 255, 0],
            Color::Pink => [255, 192, 203],
            Color::Orange => [255, 165, 0],
            Color::Yellow => [255, 255, 0],
            Color::Black => [0, 0, 0],
            Color::White => [255, 255, 255],
            Color::Purple => [128, 0, 128],
            Color::Brown => [165, 42, 42],
            Color::Cyan => [0, 255, 255],
            Color::Lime => [0, 255, 0],
            Color::Maroon => [128, 0, 0],
            Color::Rose => [255, 0, 127],
            Color::Banana => [255, 225, 53],
            Color::Gray => [128, 128, 128],
            Color::Tan => [210, 180, 140],
            Color::Coral => [255, 127, 80],
        }
    }

    pub fn to_egui_color(&self) -> egui::Color32 {
        let rgb = self.to_rgb();
        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
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
    pub user_id: usize, // Userのインデックス参照
    pub start: Point,
    pub points: Vec<Point>,
}

impl Route {
    pub fn new(user_id: usize, start: Point) -> Self {
        Self {
            user_id,
            start,
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
}

impl Wave {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            killed: None,
            kill_location: None,
            notes: String::new(),
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

    pub fn current_wave(&mut self) -> Option<&mut Wave> {
        self.waves.last_mut()
    }

    pub fn get_wave(&self, index: usize) -> Option<&Wave> {
        self.waves.get(index)
    }

    pub fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }
}
