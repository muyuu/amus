use crate::features::location::LocationInteraction;
use crate::models::Route;
use crate::state::AppState;

pub struct RouteDrawingInteraction;

impl RouteDrawingInteraction {
    pub fn handle(state: &mut AppState, response: &egui::Response) {
        let user_id = match state.selected_user_id() {
            Some(user_id) => user_id,
            None => return,
        };

        // マウス操作の処理（常にフリーハンド描画）
        if response.drag_started() {
            Self::handle_drag_start(response, state, user_id);
        } else if response.dragged() {
            Self::handle_dragging(response, state, user_id);
        }
    }

    fn handle_drag_start(response: &egui::Response, state: &mut AppState, user_id: usize) {
        // 開始地点・終了時地点からドラッグ開始した場合は無視する
        if LocationInteraction::detect_drag_start(state, response).is_some() {
            return;
        };

        // ドラッグ開始時に新しいルートを作成
        if response.interact_pointer_pos().is_some() {
            let route = Route::new(user_id);
            state.push_route(route);
        }

        // ドラッグ開始で line を追加
        if let Ok(mut wave) = state.current_wave_mut() {
            if let Some(route) = wave.routes.iter_mut().find(|r| r.user_id == user_id) {
                route.add_line(vec![]);
            }
        }
    }

    fn handle_dragging(response: &egui::Response, state: &mut AppState, user_id: usize) {
        if LocationInteraction::detect_drag_start(state, response).is_some() {
            return;
        };

        let pos = match response.interact_pointer_pos() {
            Some(p) => p,
            None => return,
        };

        let rect = response.rect;
        let point = LocationInteraction::screen_to_normalized_point(pos, rect);

        // このユーザーのルートが存在しない場合は作成（念のため）
        if let Ok(mut wave) = state.current_wave_mut() {
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
