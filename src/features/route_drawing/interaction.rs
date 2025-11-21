use crate::features::location::LocationInteraction;
use crate::models::{Route, Wave};

pub struct RouteDrawingInteraction;

impl RouteDrawingInteraction {
    /// フリーハンド描画の処理
    pub fn handle_freehand_drawing(response: &egui::Response, wave: &mut Wave, user_id: usize) {
        if response.drag_started() {
            Self::handle_drag_start(response, wave, user_id);
        } else if response.dragged() {
            Self::handle_dragging(response, wave, user_id);
        }
    }

    fn handle_drag_start(response: &egui::Response, wave: &mut Wave, user_id: usize) {
        // ドラッグ開始時に新しいルートを作成
        if response.interact_pointer_pos().is_some() {
            let route = Route::new(user_id);
            wave.routes.push(route);
        }

        // ドラッグ開始で line を追加
        if let Some(route) = wave.routes.iter_mut().find(|r| r.user_id == user_id) {
            route.add_line(vec![]);
        }
    }

    fn handle_dragging(response: &egui::Response, wave: &mut Wave, user_id: usize) {
        let pos = match response.interact_pointer_pos() {
            Some(p) => p,
            None => return,
        };

        let rect = response.rect;
        let point = LocationInteraction::screen_to_normalized_point(pos, rect);

        // このユーザーのルートが存在しない場合は作成（念のため）
        if let Some(route) = wave.routes.iter_mut().find(|r| r.user_id == user_id) {
            route.add_point(point);
        } else {
            // ルートが存在しない場合は新規作成
            let mut route = Route::new(user_id);
            route.add_point(point);
            wave.routes.push(route);
        }
    }
}
