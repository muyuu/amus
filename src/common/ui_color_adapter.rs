use egui::Color32;

use crate::models::Color;

pub fn to_egui_color(color: &Color) -> Color32 {
    let rgb = color.to_rgb();
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}
