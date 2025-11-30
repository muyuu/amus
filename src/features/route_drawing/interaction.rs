use egui::*;

use crate::features::location::LocationInteraction;
use crate::models::route::{Draw, Erase};
use crate::models::Route;
use crate::state::AppState;

pub struct RouteDrawingInteraction;

impl RouteDrawingInteraction {
    pub fn handle(state: &mut AppState, response: &Response) {
        // マウス操作の処理（常にフリーハンド描画）
        if response.drag_started() {
            Self::handle_drag_start(state, response);
        } else if response.dragged() {
            Self::handle_dragging(state, response);
        }
    }

    fn handle_drag_start(state: &mut AppState, response: &Response) {
        // 開始地点・終了時地点からドラッグ開始した場合は無視する
        if LocationInteraction::detect_drag_start(state, response).is_some() {
            return;
        };

        let _ = match response.interact_pointer_pos() {
            Some(p) => p,
            None => return,
        };

        // ドラッグ開始時に新しいルートを作成
        let route = if state.erase_mode() {
            Route::Erase(Erase::new())
        } else {
            let player_id = match state.selected_player_id() {
                Some(player_id) => player_id,
                None => return,
            };
            Route::Draw(Draw::new(player_id))
        };
        state.push_route(route);
    }

    fn handle_dragging(state: &mut AppState, response: &Response) {
        if LocationInteraction::detect_drag_start(state, response).is_some() {
            return;
        };

        let pos = match response.interact_pointer_pos() {
            Some(p) => p,
            None => return,
        };

        let rect = response.rect;
        let point = LocationInteraction::screen_to_normalized_point(pos, rect);

        let _ = match state.current_wave() {
            Ok(wave) => wave,
            _ => return,
        };

        let mut l = match state.last_route_mut() {
            Some(route) => route,
            None => return,
        };

        l.add_point(point);
    }
}
