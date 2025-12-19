use super::Actions;
use crate::models::route::{Draw, Erase};
use crate::models::{Point, Route};

/// ルート描画機能から発行されるアクション
#[derive(Debug, Clone)]
pub enum RouteDrawingAction {
    /// 新しいルートを開始（描画モード）
    StartDraw,
    /// 新しいルートを開始（消しゴムモード）
    StartErase,
    /// ルートにポイントを追加
    AddPoint(Point),
}

impl Actions<'_> {
    pub fn handle_route_drawing(&mut self, action: RouteDrawingAction) {
        match action {
            RouteDrawingAction::StartDraw => {
                let player_id = match self.state.selected_player_id() {
                    Some(id) => id,
                    None => return,
                };
                self.state.push_route(Route::Draw(Draw::new(player_id)));
            }
            RouteDrawingAction::StartErase => {
                self.state.push_route(Route::Erase(Erase::default()));
            }
            RouteDrawingAction::AddPoint(point) => {
                if self.state.current_wave().is_ok() {
                    self.state.add_point_to_last_route(point);
                }
            }
        }
    }
}
