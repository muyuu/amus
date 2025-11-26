use crate::{models::{Route, route::Draw}, state::AppState};
use egui::*;

pub struct RouteDrawingView;

impl RouteDrawingView {
    pub fn render(state: &AppState, response: &egui::Response, ui: &mut egui::Ui) {

        let routes = match state.routes() {
            Some(routes) => routes,
            None => return,
        };

        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        for route in routes {
            match route {
                Route::Draw(draw) => Self::draw_route(state, &draw, &painter, rect),
                Route::Erase(_) => Self::erase_route(),
            }
        }
    }

    /// ルートを描画（開始地点と軌跡）Response
    fn draw_route(state: &AppState, route: &Draw, painter: &Painter, rect: Rect) {
        let game = match state.game() {
            Some(game) => game,
            None => return,
        };

        let user = match game.get_users().get(route.user_id) {
            Some(user) => user,
            None => return,
        };

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
                painter.line_segment([prev_pos, pos], (stroke_width, user.color.to_egui_color()));
                prev_pos = pos;
            }
        }
    }

    fn erase_route() {
        // 消去ルートの描画ロジックをここに実装
    }
}
