use crate::common::ui_color_adapter::to_egui_color;
use crate::models::{
    route::{Draw, Erase},
    Point, Route,
};
use crate::state::Slices;
use egui::*;

pub struct RouteDrawingView;

impl RouteDrawingView {
    pub fn render(slices: &Slices<'_>, response: &Response, ui: &mut Ui) {
        let wave_slice = slices.wave();
        let routes = match wave_slice.routes() {
            Some(routes) => {
                if routes.is_empty() {
                    return;
                }
                routes
            }
            None => return,
        };

        // 消去処理を適用した新しいRouteを生成
        let processed_routes = Self::process_routes_with_eraser(routes);

        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        // 処理済みのRouteを描画（Drawのみ）
        for route in &processed_routes {
            if let Route::Draw(draw) = route {
                Self::draw_route(slices, draw, &painter, rect);
            }
        }
    }

    /// route を走査して実際に描画する Route を新たに作成する
    /// Erase で登録された Point はそれより以前に登録された Draw の描画を削除する必要がある
    /// - Draw の場合は processed_routes に追加
    /// - Erase の場合は processed_routes を走査して消去範囲に含まれる Point を削除
    fn process_routes_with_eraser(routes: &[Route]) -> Vec<Route> {
        let mut processed_routes: Vec<Route> = Vec::new();

        for route in routes {
            match route {
                Route::Draw(draw) => {
                    // Drawの場合はそのまま追加
                    processed_routes.push(Route::Draw(draw.clone()));
                }
                Route::Erase(erase) => {
                    // Eraseの場合は、これまでに追加されたDrawから消去範囲に含まれるPointを削除
                    Self::apply_erase(&mut processed_routes, erase);
                }
            }
        }

        processed_routes
    }

    /// Eraseを適用して、processed_routesから消去範囲に含まれるPointを削除
    fn apply_erase(processed_routes: &mut [Route], erase: &Erase) {
        let erase_radius = 10.0; // 消去半径（ピクセル単位で正規化座標に変換される）

        // Eraseの全てのポイントを収集
        let erase_points: Vec<&Point> = erase.lines.iter().flat_map(|line| line.iter()).collect();

        if erase_points.is_empty() {
            return;
        }

        // processed_routesを走査して、消去範囲に含まれるPointで線を分割
        for route in processed_routes.iter_mut() {
            if let Route::Draw(draw) = route {
                let mut new_lines: Vec<Vec<Point>> = Vec::new();

                for line in draw.lines.iter() {
                    // 各lineを処理して、消去範囲に含まれるPointで分割
                    let split_lines = Self::split_line_by_erase(line, &erase_points, erase_radius);
                    new_lines.extend(split_lines);
                }

                draw.lines = new_lines;
            }
        }
    }

    /// lineを消去範囲に含まれるPointで分割
    fn split_line_by_erase(
        line: &[Point],
        erase_points: &[&Point],
        radius: f32,
    ) -> Vec<Vec<Point>> {
        let mut result: Vec<Vec<Point>> = vec![];
        let mut current_line: Vec<Point> = vec![];

        for point in line {
            if Self::is_point_in_erase_range(point, erase_points, radius) {
                // 消去範囲に含まれる場合
                if !current_line.is_empty() {
                    // 現在のlineを保存して新しいlineを開始
                    result.push(current_line);
                    current_line = Vec::new();
                }
                // このポイントは追加しない（消去される）
            } else {
                // 消去範囲外の場合は現在のlineに追加
                current_line.push(point.clone());
            }
        }

        // 最後に残ったlineを追加
        if !current_line.is_empty() {
            result.push(current_line);
        }

        result
    }

    /// ポイントが消去範囲に含まれるかチェック
    fn is_point_in_erase_range(point: &Point, erase_points: &[&Point], radius: f32) -> bool {
        for erase_point in erase_points {
            let dx = point.x - erase_point.x;
            let dy = point.y - erase_point.y;
            let distance_squared = dx * dx + dy * dy;

            // 正規化座標なので、半径も正規化する必要がある
            // 仮に画面サイズを1000pxとして、20pxの半径は0.02
            let normalized_radius = radius / 1000.0;

            if distance_squared <= normalized_radius * normalized_radius {
                return true;
            }
        }
        false
    }

    /// ルートを描画（開始地点と軌跡）Response
    fn draw_route(slices: &Slices<'_>, route: &Draw, painter: &Painter, rect: Rect) {
        if route.lines.is_empty() {
            return;
        }

        let player_slice = slices.player();
        let player = match player_slice.player(route.player_id) {
            Some(p) => p,
            None => return,
        };

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
                painter.line_segment(
                    [prev_pos, pos],
                    (stroke_width, to_egui_color(&player.color)),
                );
                prev_pos = pos;
            }
        }
    }
}
