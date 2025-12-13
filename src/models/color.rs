use egui::*;
use serde::{Deserialize, Serialize};

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

    pub fn to_egui_color(&self) -> Color32 {
        let rgb = self.to_rgb();
        Color32::from_rgb(rgb[0], rgb[1], rgb[2])
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

    pub fn name_lowercase(&self) -> String {
        self.name().to_lowercase()
    }
}
