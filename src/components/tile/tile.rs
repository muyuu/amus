use egui::*;

use super::base::{base, TileBaseConf, TileResult};
use super::constants::TileConstants;

#[allow(dead_code)]
pub struct TileConf {
    pub color: Color32,
    pub label: Option<String>,
    pub size: Option<Vec2>,
    pub is_selected: bool,
    pub is_dragging: bool,
}

pub fn tile(ui: &mut Ui, conf: &TileConf) -> TileResult {
    let c = TileBaseConf {
        color: conf.color,
        label: conf.label.clone(),
        size: if let Some(size) = conf.size {
            size
        } else {
            Vec2::new(TileConstants::SIZE, TileConstants::SIZE)
        },
        is_selected: conf.is_selected,
        is_dragging: conf.is_dragging,
    };
    base(ui, &c)
}
