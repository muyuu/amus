use crate::models::{Point, Route};
use egui::*;

pub struct RouteDrawingView;

impl RouteDrawingView {
    /// ルートを描画（開始地点と軌跡）
    pub fn draw_route(
        painter: &egui::Painter,
        route: &Route,
        spawn_location: Option<&Point>,
        color: Color32,
        rect: Rect,
    ) {
        // 開始地点を描画（spawn_locationsから取得、マーカーのみ）
        if let Some(spawn_point) = spawn_location {
            let pos = pos2(
                rect.min.x + spawn_point.x * rect.size().x,
                rect.min.y + spawn_point.y * rect.size().y,
            );
            painter.circle_filled(pos, 8.0, color);
        }

        // 軌跡を描画（route.pointsの各ポイント間のみ）
        if route.lines.is_empty() {
            return;
        }

        for line in route.lines.iter() {
            let mut prev_pos = if let Some(first_point) = line.first() {
                pos2(
                    rect.min.x + first_point.x * rect.size().x,
                    rect.min.y + first_point.y * rect.size().y,
                )
            } else {
                return;
            };

            // 最初のポイントから線を引く（spawn_locationからではない）
            for point in line.iter().skip(1) {
                let pos = pos2(
                    rect.min.x + point.x * rect.size().x,
                    rect.min.y + point.y * rect.size().y,
                );
                painter.line_segment([prev_pos, pos], (2.0, color));
                prev_pos = pos;
            }
        }
    }
}
