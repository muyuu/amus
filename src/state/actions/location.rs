use super::Actions;
use crate::models::color::Color;
use crate::models::player::PlayerId;
use crate::models::{DraggingLocation, LocationType, Point};

/// 位置関連機能から発行されるアクション
#[derive(Debug, Clone)]
pub enum LocationAction {
    /// ドラッグ中の位置を設定
    SetDraggingLocation(Option<DraggingLocation>),
    /// 位置を追加
    AddLocation(LocationType, PlayerId, Point),
    /// 位置を削除（メニューから）
    DeleteLocation(LocationType, PlayerId),
    /// ドラッグ終了時の処理
    StopDragging(LocationType, PlayerId),
    /// プレイヤー状態をトグル
    TogglePlayerState(PlayerId),
    /// プレイヤーの色を変更（重複時はスワップ）
    ForceUpdatePlayerColor(PlayerId, Color),
    /// エリアクリック時の処理
    HandleAreaClick(Point),
    /// 終了位置ドラッグ中の処理
    HandleEndLocationDragged(PlayerId, Point),
}

impl Actions<'_> {
    pub fn handle_location(&mut self, action: LocationAction) {
        match action {
            LocationAction::SetDraggingLocation(location) => {
                self.state.set_dragging_location(location);
            }
            LocationAction::AddLocation(location_type, player_id, point) => {
                self.state.add_location(location_type, player_id, point);
            }
            LocationAction::DeleteLocation(location_type, player_id) => {
                self.state.remove_location(location_type, player_id);
            }
            LocationAction::StopDragging(location_type, player_id) => {
                // ドラッグ中の位置と一致する場合のみドラッグ状態を解除
                if let Some(dragging) = self.state.dragging_location() {
                    if dragging.location_type == location_type && dragging.player_id == player_id {
                        self.state.set_dragging_location(None);
                    }
                }
            }
            LocationAction::TogglePlayerState(player_id) => {
                self.state.toggle_player_state(player_id);
            }
            LocationAction::ForceUpdatePlayerColor(player_id, color) => {
                self.state.force_update_player_color(player_id, color);
            }
            LocationAction::HandleAreaClick(point) => {
                if self.state.dragging_location().is_some() {
                    return;
                }

                let selected_player_id = match self.state.selected_player_id() {
                    Some(id) => id,
                    None => return,
                };

                // 出現位置がすでに設定されている場合は何もしない
                if let Ok(wave) = self.state.current_wave() {
                    if wave.spawn_locations.contains_key(&selected_player_id) {
                        return;
                    }
                }

                self.state
                    .add_location(LocationType::Spawn, selected_player_id, point);
            }
            LocationAction::HandleEndLocationDragged(player_id, point) => {
                self.state.add_location(LocationType::End, player_id, point);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PlayerState;
    use crate::state::{Actions, AppState};

    fn state_with_player() -> (AppState, PlayerId) {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let id = state.players().unwrap()[0].id;
        (state, id)
    }

    #[test]
    fn add_location_registers_point() {
        let (mut state, id) = state_with_player();

        Actions::new(&mut state).handle_location(LocationAction::AddLocation(
            LocationType::Spawn,
            id,
            Point::new(3.0, 4.0),
        ));

        let spawns = state.locations(LocationType::Spawn).unwrap();
        let point = spawns.get(&id).expect("出現位置が登録される");
        assert_eq!((point.x, point.y), (3.0, 4.0));
    }

    #[test]
    fn delete_location_removes_point() {
        let (mut state, id) = state_with_player();
        Actions::new(&mut state).handle_location(LocationAction::AddLocation(
            LocationType::Spawn,
            id,
            Point::new(1.0, 1.0),
        ));

        Actions::new(&mut state)
            .handle_location(LocationAction::DeleteLocation(LocationType::Spawn, id));

        assert!(state.locations(LocationType::Spawn).unwrap().is_empty());
    }

    #[test]
    fn toggle_player_state_advances_to_killed() {
        let (mut state, id) = state_with_player();

        Actions::new(&mut state).handle_location(LocationAction::TogglePlayerState(id));

        assert_eq!(state.player(id).unwrap().state, PlayerState::Killed);
    }

    #[test]
    fn area_click_adds_spawn_for_selected_player() {
        let (mut state, id) = state_with_player();
        state.set_selected_player_id(Some(id));

        Actions::new(&mut state)
            .handle_location(LocationAction::HandleAreaClick(Point::new(5.0, 5.0)));

        let spawns = state.locations(LocationType::Spawn).unwrap();
        assert!(spawns.contains_key(&id));
    }

    #[test]
    fn area_click_without_selection_does_nothing() {
        let (mut state, _id) = state_with_player();
        // プレイヤー未選択

        Actions::new(&mut state)
            .handle_location(LocationAction::HandleAreaClick(Point::new(5.0, 5.0)));

        assert!(state.locations(LocationType::Spawn).unwrap().is_empty());
    }
}
