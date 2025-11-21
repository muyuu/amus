use crate::models::{user, Game, Wave};
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
                let size = 28.0;
                let stroke_width = 3.0;
                painter.rect_filled(
                    Rect::from_center_size(pos, egui::Vec2::new(size, size)),
                    stroke_width,
                    color,
                );
                painter.rect_stroke(
                    Rect::from_center_size(pos, egui::Vec2::new(size, size)),
                    stroke_width,
                    (stroke_width, Color32::WHITE),
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
                let size = 16.0;
                let stroke_width = 3.0;

                painter.circle_filled(pos, size, color);
                painter.circle_stroke(pos, size, (stroke_width, Color32::WHITE));

                Self::render_dead_mark(user, painter, pos, size);
            }
        }
    }

    // 死亡している場合はバツ印を描画
    fn render_dead_mark(user: &user::User, painter: &egui::Painter, pos: Pos2, size: f32) {
        if user.alive {
            return;
        }

        let cross_size = size; // 丸の直径と同じサイズ
        let cross_width = 6.0;

        // 左上から右下への線
        painter.line_segment(
            [
                pos2(pos.x - cross_size, pos.y - cross_size),
                pos2(pos.x + cross_size, pos.y + cross_size),
            ],
            (cross_width, Color32::BLACK),
        );

        // 右上から左下への線
        painter.line_segment(
            [
                pos2(pos.x + cross_size, pos.y - cross_size),
                pos2(pos.x - cross_size, pos.y + cross_size),
            ],
            (cross_width, Color32::BLACK),
        );
    }
}
