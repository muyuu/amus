use crate::models::{Game, Wave};
use egui::*;

pub struct LocationView;

impl LocationView {
    /// 出現位置と終了時位置を描画
    pub fn render(painter: &egui::Painter, game: &Game, wave: &Wave, rect: Rect) {
        // 出現場所を描画（四角）
        for (user_id, point) in &wave.spawn_locations {
            if let Some(user) = game.users.get(*user_id) {
                let color = user.color.to_egui_color();
                let pos = pos2(
                    rect.min.x + point.x * rect.size().x,
                    rect.min.y + point.y * rect.size().y,
                );
                let size = 12.0;
                painter.rect_filled(
                    Rect::from_center_size(pos, egui::Vec2::new(size, size)),
                    2.0,
                    color,
                );
                painter.rect_stroke(
                    Rect::from_center_size(pos, egui::Vec2::new(size, size)),
                    2.0,
                    (2.0, Color32::WHITE),
                );
            }
        }

        // 終了時位置を描画（丸）
        for (user_id, point) in &wave.end_locations {
            if let Some(user) = game.users.get(*user_id) {
                let color = user.color.to_egui_color();
                let pos = pos2(
                    rect.min.x + point.x * rect.size().x,
                    rect.min.y + point.y * rect.size().y,
                );
                painter.circle_filled(pos, 8.0, color);
                painter.circle_stroke(pos, 8.0, (2.0, Color32::WHITE));
            }
        }
    }
}
