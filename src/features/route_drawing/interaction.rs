use crate::features::location::LocationInteraction;
use crate::models::{Point, Route, Wave};
use crate::state::DrawingMode;

pub struct RouteDrawingInteraction;

impl RouteDrawingInteraction {
    /// マップのインタラクション処理（描画モードに応じて処理を分岐）
    pub fn handle_map_interaction(
        response: &egui::Response,
        wave: &mut Wave,
        user_id: usize,
        drawing_mode: DrawingMode,
    ) {
        match drawing_mode {
            DrawingMode::ClickToLine => {
                Self::handle_click_to_line(response, wave, user_id);
            }
            DrawingMode::Freehand => {
                Self::handle_freehand_drawing(response, wave, user_id);
            }
            DrawingMode::None => {}
        }
    }

    /// クリックで線を描くモードの処理
    fn handle_click_to_line(response: &egui::Response, wave: &mut Wave, user_id: usize) {
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rect = response.rect;
                let point = LocationInteraction::screen_to_normalized_point(pos, rect);

                // 既存のルートを探すか、新規作成
                if let Some(route) = wave.routes.iter_mut().find(|r| r.user_id == user_id) {
                    // 既存のルートがある場合はポイントを追加
                    route.add_point(point);
                } else {
                    // 新規ルートを作成
                    let mut route = Route::new(user_id);
                    route.add_point(point);
                    wave.routes.push(route);
                }
            }
        }
    }

    /// フリーハンド描画の処理
    fn handle_freehand_drawing(response: &egui::Response, wave: &mut Wave, user_id: usize) {
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

    /// 一時的なポイントを更新（フリーハンド描画中のプレビュー用）
    pub fn update_temp_points(
        temp_points: &mut Vec<Point>,
        response: &egui::Response,
        point: Point,
    ) {
        if response.drag_started() {
            temp_points.clear();
            temp_points.push(point);
        } else if response.dragged() {
            temp_points.push(point);
        }
    }
}
