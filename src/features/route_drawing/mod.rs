mod view;

use crate::app_action::AppAction;
use crate::common::current_point_with_response;
use crate::state::{AppState, RouteDrawingAction, Slices};
use egui::{Response, Ui};
use view::RouteDrawingView;

pub struct RouteDrawingFeature;

impl RouteDrawingFeature {
    pub fn render(state: &AppState, response: &Response, ui: &mut Ui) -> Vec<AppAction> {
        // ViewにはSlices（読み取り専用）を渡す
        let slices = state.slices();

        // 描画
        RouteDrawingView::render(&slices, response, ui);

        // ユーザー操作を検出して Action として返す
        Self::detect_actions(&slices, response)
            .into_iter()
            .map(AppAction::RouteDrawing)
            .collect()
    }

    fn detect_actions(slices: &Slices<'_>, response: &Response) -> Vec<RouteDrawingAction> {
        let mut actions = Vec::new();

        let ui_slice = slices.ui();
        let player_slice = slices.player();

        if response.drag_started() {
            if response.interact_pointer_pos().is_some() {
                if ui_slice.erase_mode() {
                    actions.push(RouteDrawingAction::StartErase);
                } else if player_slice.selected_player_id().is_some() {
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
