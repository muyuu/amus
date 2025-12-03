use egui::*;

use crate::common::current_point_with_response;
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
        let point = match current_point_with_response(response, response.rect) {
            Some(p) => p,
            None => return,
        };

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
