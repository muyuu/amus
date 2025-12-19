mod view;

use crate::common::current_point_with_response;
use crate::state::{Actions, AppState, RouteDrawingAction};
use egui::{Response, Ui};
use view::RouteDrawingView;

pub struct RouteDrawingFeature;

impl RouteDrawingFeature {
    pub fn render(state: &mut AppState, response: &Response, ui: &mut Ui) {
        // 描画
        RouteDrawingView::render(state, response, ui);

        // ユーザー操作を検出してActionsで処理
        let route_actions = Self::detect_actions(state, response);
        let mut actions = Actions::new(state);
        for action in route_actions {
            actions.handle_route_drawing(action);
        }
    }

    fn detect_actions(state: &AppState, response: &Response) -> Vec<RouteDrawingAction> {
        let mut actions = Vec::new();

        if response.drag_started() {
            if response.interact_pointer_pos().is_some() {
                if state.erase_mode() {
                    actions.push(RouteDrawingAction::StartErase);
                } else if state.selected_player_id().is_some() {
                    actions.push(RouteDrawingAction::StartDraw);
                }
            }
        } else if response.dragged() {
            if let Some(point) = current_point_with_response(response, response.rect) {
                actions.push(RouteDrawingAction::AddPoint(point));
            }
        }

        actions
    }
}
