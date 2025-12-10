use egui::*;

use crate::common::choose_text_color;

use super::constants::TileConstants;
use super::tile;
use super::TileConf;
use super::TileResult;

pub struct WithMarkConf {
    pub color: Color32,
    pub label: Option<String>,
    pub size: Option<Vec2>,
    pub is_selected: bool,
    pub is_dragging: bool,
    pub marked: bool,
}
pub fn with_mark(ui: &mut Ui, conf: &WithMarkConf) -> TileResult {
    let r = tile(
        ui,
        &TileConf {
            color: conf.color,
            label: conf.label.clone(),
            size: conf.size,
            is_selected: conf.is_selected,
            is_dragging: conf.is_dragging,
        },
    );

    // resolve_commsがtrueの場合、中抜きの丸を表示
    if conf.marked {
        let center = r.rect.center();
        let radius = r.rect.width().min(r.rect.height()) * 0.3;

        // の色が赤の場合は白、それ以外は赤
        let circle_color = choose_text_color(conf.color);

        ui.painter().circle_stroke(
            center,
            radius,
            Stroke::new(TileConstants::MARK_SIZE, circle_color),
        );
    }

    r
}
