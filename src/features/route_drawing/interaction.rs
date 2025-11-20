use crate::features::location::LocationInteraction;
use crate::models::{Point, Route, Wave};

pub struct RouteDrawingInteraction;

impl RouteDrawingInteraction {
    /// フリーハンド描画の処理
    pub fn handle_freehand_drawing(response: &egui::Response, wave: &mut Wave, user_id: usize) {
        if response.drag_started() {
            // ドラッグ開始時に新しいルートを作成
            // このユーザーの既存のルートをすべて削除（1ユーザー1本にするため）
            wave.routes.retain(|r| r.user_id != user_id);

            if response.interact_pointer_pos().is_some() {
                let route = Route::new(user_id);
                wave.routes.push(route);
            }
        } else if response.dragged() {
            // ドラッグ中はポイントを追加
            if let Some(pos) = response.interact_pointer_pos() {
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
    }
}
