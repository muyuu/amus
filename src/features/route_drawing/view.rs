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
        // 開始地点を描画（spawn_locationsから取得）
        let mut prev_pos = if let Some(spawn_point) = spawn_location {
            let pos = pos2(
                rect.min.x + spawn_point.x * rect.size().x,
                rect.min.y + spawn_point.y * rect.size().y,
            );
            painter.circle_filled(pos, 8.0, color);
            pos
        } else if let Some(first_point) = route.points.first() {
            // spawn_locationがない場合は最初のポイントを開始地点として使用
            let pos = pos2(
                rect.min.x + first_point.x * rect.size().x,
                rect.min.y + first_point.y * rect.size().y,
            );
            painter.circle_filled(pos, 8.0, color);
            pos
        } else {
            return; // ポイントがない場合は描画しない
        };

        // 軌跡を描画
        for point in &route.points {
            let pos = pos2(
                rect.min.x + point.x * rect.size().x,
                rect.min.y + point.y * rect.size().y,
            );
            painter.line_segment([prev_pos, pos], (2.0, color));
            prev_pos = pos;
        }
    }

    /// 一時的なルート（描画中）を描画
    pub fn draw_temp_route(painter: &egui::Painter, points: &[Point], color: Color32, rect: Rect) {
        if points.is_empty() {
            return;
        }

        let mut prev_pos = pos2(
            rect.min.x + points[0].x * rect.size().x,
            rect.min.y + points[0].y * rect.size().y,
        );

        for point in points.iter().skip(1) {
            let pos = pos2(
                rect.min.x + point.x * rect.size().x,
                rect.min.y + point.y * rect.size().y,
            );
            painter.line_segment([prev_pos, pos], (2.0, color));
            prev_pos = pos;
        }
    }
}
