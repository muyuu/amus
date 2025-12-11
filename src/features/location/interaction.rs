use crate::common::current_point_with_ui;
use crate::features::location::view::LocationViewResult;
use crate::models::*;
use crate::state::AppState;
use egui::*;

pub struct LocationInteraction;

impl LocationInteraction {
    pub fn handle(
        state: &mut AppState,
        result: &LocationViewResult,
        response: &Response,
        ui: &mut Ui,
    ) {
        let point = match current_point_with_ui(ui) {
            Some(p) => p,
            None => return,
        };

        // dragging_player_id が Some の場合は終了時位置をドラッグしている
        // その間は常に end_location を更新する
        if let Some(player_id) = state.dragging_player_id() {
            Self::handle_end_location_dragged(state, ui, player_id);
        }

        if response.clicked() {
            Self::handle_area_click(state, ui);
        }

        // 出現位置ドラッグ開始
        if let Some(player_id) = result.spawn.drag_start_player_id {
            state.set_dragging_location(Some(DraggingLocation {
                location_type: LocationType::Spawn,
                player_id,
            }));
        }

        // 出現位置ドラッグ中
        if let Some(player_id) = result.spawn.dragging_player_id {
            Self::add_location(state, LocationType::Spawn, player_id, point.clone());
        }

        // 出現位置ドラッグ終了
        if let Some(player_id) = result.spawn.drag_stop_player_id {
            Self::remove_location(state, player_id, LocationType::Spawn);
        }

        // 出現位置クリック
        if let Some(_player_id) = result.spawn.click_player_id {
            // TOOD: 削除とかしたい
        }

        // 終了時位置ドラッグ開始
        if let Some(player_id) = result.end.drag_start_player_id {
            state.set_dragging_location(Some(DraggingLocation {
                location_type: LocationType::End,
                player_id,
            }));
        }

        // 終了時位置ドラッグ中
        if let Some(player_id) = result.end.dragging_player_id {
            Self::add_location(state, LocationType::End, player_id, point.clone());
        }

        // 終了時位置ドラッグ終了
        if let Some(player_id) = result.end.drag_stop_player_id {
            Self::remove_location(state, player_id, LocationType::End);
        }

        // 終了時位置クリック
        if let Some(player_id) = result.end.click_player_id {
            state.toggle_player_state(player_id);
        }
    }

    fn handle_end_location_dragged(state: &mut AppState, ui: &mut Ui, player_id: PlayerId) {
        let point = match current_point_with_ui(ui) {
            Some(p) => p,
            None => return,
        };
        state.add_end_location(player_id, point);
    }

    fn handle_area_click(state: &mut AppState, ui: &mut Ui) {
        if state.dragging_location().is_some() {
            return;
        }

        let selected_player_id = match state.selected_player_id() {
            Some(id) => id,
            None => return,
        };

        let point = match current_point_with_ui(ui) {
            Some(p) => p,
            None => return,
        };

        // 出現位置がすでに設定されている場合は何もしない
        // current_wave を一時的に借用して確認するためブロックスコープに入る
        {
            let wave = match state.current_wave() {
                Ok(wave) => wave,
                _ => return,
            };

            if wave.spawn_locations.contains_key(&selected_player_id) {
                return;
            }
        }

        Self::add_location(state, LocationType::Spawn, selected_player_id, point);
    }

    fn add_location(state: &mut AppState, location_type: LocationType, id: PlayerId, point: Point) {
        match location_type {
            LocationType::Spawn => {
                state.add_spawn_location(id, point);
            }
            LocationType::End => {
                state.add_end_location(id, point);
            }
        }
    }

    fn remove_location(state: &mut AppState, player_id: PlayerId, location_type: LocationType) {
        let dragging_location = match state.dragging_location() {
            Some(dragging_location) => dragging_location,
            None => return,
        };

        if dragging_location.location_type != location_type {
            return;
        }

        if dragging_location.player_id != player_id {
            return;
        }

        state.set_dragging_location(None);
    }
}
