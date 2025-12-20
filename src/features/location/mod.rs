mod constants;
mod view;

use crate::common::current_point_with_ui;
use crate::models::{DraggingLocation, LocationType};
use crate::state::{Actions, AppState, LocationAction};
use egui::{Response, Ui};
use view::LocationView;

pub struct LocationFeature;

impl LocationFeature {
    pub fn render(state: &mut AppState, response: &Response, ui: &mut Ui) {
        // ViewにはSlices（読み取り専用）を渡す
        let slices = state.slices();
        let result = LocationView::render(&slices, ui);

        // ユーザー操作をActionsに変換
        let mut location_actions = Vec::new();

        // dragging_player_id が Some の場合は終了時位置をドラッグしている
        if let Some(player_id) = slices.ui().dragging_player_id() {
            if let Some(point) = current_point_with_ui(ui) {
                location_actions.push(LocationAction::HandleEndLocationDragged(player_id, point));
            }
        }

        // エリアクリック
        if response.clicked() {
            if let Some(point) = current_point_with_ui(ui) {
                location_actions.push(LocationAction::HandleAreaClick(point));
            }
        }

        // 出現位置ドラッグ開始
        if let Some(player_id) = result.spawn.drag_start_player_id {
            location_actions.push(LocationAction::SetDraggingLocation(Some(
                DraggingLocation {
                    location_type: LocationType::Spawn,
                    player_id,
                },
            )));
        }

        // 出現位置ドラッグ中
        if let Some(player_id) = result.spawn.dragging_player_id {
            if let Some(point) = current_point_with_ui(ui) {
                location_actions.push(LocationAction::AddLocation(
                    LocationType::Spawn,
                    player_id,
                    point,
                ));
            }
        }

        // 出現位置ドラッグ終了
        if let Some(player_id) = result.spawn.drag_stop_player_id {
            location_actions.push(LocationAction::StopDragging(LocationType::Spawn, player_id));
        }

        // 出現位置の色変更
        if let Some((player_id, color)) = result.spawn.color_change {
            location_actions.push(LocationAction::ForceUpdatePlayerColor(player_id, color));
        }

        // 出現位置の削除
        if let Some(player_id) = result.spawn.delete_player_id {
            location_actions.push(LocationAction::DeleteLocation(
                LocationType::Spawn,
                player_id,
            ));
        }

        // 終了時位置ドラッグ開始
        if let Some(player_id) = result.end.drag_start_player_id {
            location_actions.push(LocationAction::SetDraggingLocation(Some(
                DraggingLocation {
                    location_type: LocationType::End,
                    player_id,
                },
            )));
        }

        // 終了時位置ドラッグ中
        if let Some(player_id) = result.end.dragging_player_id {
            if let Some(point) = current_point_with_ui(ui) {
                location_actions.push(LocationAction::AddLocation(
                    LocationType::End,
                    player_id,
                    point,
                ));
            }
        }

        // 終了時位置ドラッグ終了
        if let Some(player_id) = result.end.drag_stop_player_id {
            location_actions.push(LocationAction::StopDragging(LocationType::End, player_id));
        }

        // 終了時位置クリック
        if let Some(player_id) = result.end.click_player_id {
            location_actions.push(LocationAction::TogglePlayerState(player_id));
        }

        // 終了時位置の色変更
        if let Some((player_id, color)) = result.end.color_change {
            location_actions.push(LocationAction::ForceUpdatePlayerColor(player_id, color));
        }

        // 終了時位置の削除
        if let Some(player_id) = result.end.delete_player_id {
            location_actions.push(LocationAction::DeleteLocation(LocationType::End, player_id));
        }

        // Actionsで処理
        let mut actions = Actions::new(state);
        for action in location_actions {
            actions.handle_location(action);
        }
    }
}
