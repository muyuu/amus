use crate::{models::Route, state::AppState};
use egui::*;

pub struct RouteDrawingView;

impl RouteDrawingView {
    pub fn render(state: &AppState, response: &egui::Response, ui: &mut egui::Ui) {
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        let routes = match state.routes() {
            Some(routes) => routes,
            None => return,
        };

        let game = match state.game() {
            Some(game) => game,
            None => return,
        };

        for route in routes {
            if let Some(user) = game.users.get(route.user_id) {
                let color = user.color.to_egui_color();
                Self::draw_route(&painter, &route, color, rect);
            }
        }
    }

    /// ルートを描画（開始地点と軌跡）
    fn draw_route(painter: &egui::Painter, route: &Route, color: Color32, rect: Rect) {
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
                let stroke_width = 6.0;
                painter.line_segment([prev_pos, pos], (stroke_width, color));
                prev_pos = pos;
            }
        }
    }
}
