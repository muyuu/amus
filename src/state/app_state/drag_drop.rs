use std::collections::HashMap;

use crate::models::{DraggingLocation, LocationType, PlayerId, Point};

use super::AppState;

// ドラッグ&ドロップ関連
impl AppState {
    // 読み取りは Slices に集約予定 (#136)。現状この AppState 直アクセサはテストからのみ使用。
    #[allow(dead_code)]
    pub fn dragging_player_id(&self) -> Option<PlayerId> {
        self.data.dragging_player_id
    }

    pub fn set_dragging_player_id(&mut self, player_id: Option<PlayerId>) {
        self.data.dragging_player_id = player_id;
    }

    pub fn dragging_location(&self) -> Option<DraggingLocation> {
        self.data.dragging_location
    }

    pub fn set_dragging_location(&mut self, location: Option<DraggingLocation>) {
        self.data.dragging_location = location;
    }

    // 読み取りは Slices に集約予定 (#136)。現状この AppState 直アクセサはテストからのみ使用。
    #[allow(dead_code)]
    pub fn locations(&self, location_type: LocationType) -> Option<HashMap<PlayerId, Point>> {
        self.current_wave().ok().map(|w| match location_type {
            LocationType::Spawn => w.spawn_locations.clone(),
            LocationType::End => w.end_locations.clone(),
        })
    }

    pub fn add_location(&mut self, location_type: LocationType, player_id: PlayerId, point: Point) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        match location_type {
            LocationType::Spawn => wave.spawn_locations.insert(player_id, point),
            LocationType::End => wave.end_locations.insert(player_id, point),
        };
    }

    pub fn remove_location(&mut self, location_type: LocationType, player_id: PlayerId) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        match location_type {
            LocationType::Spawn => wave.spawn_locations.remove(&player_id),
            LocationType::End => wave.end_locations.remove(&player_id),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with_player() -> (AppState, PlayerId) {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let id = state.players().unwrap()[0].id;
        (state, id)
    }

    #[test]
    fn add_location_stores_point_for_player() {
        let (mut state, id) = game_with_player();

        state.add_location(LocationType::Spawn, id, Point::new(10.0, 20.0));

        let spawns = state.locations(LocationType::Spawn).unwrap();
        let point = spawns.get(&id).expect("出現位置が登録される");
        assert_eq!((point.x, point.y), (10.0, 20.0));
    }

    #[test]
    fn add_location_keeps_spawn_and_end_separate() {
        let (mut state, id) = game_with_player();

        state.add_location(LocationType::Spawn, id, Point::new(1.0, 1.0));
        state.add_location(LocationType::End, id, Point::new(2.0, 2.0));

        assert_eq!(state.locations(LocationType::Spawn).unwrap().len(), 1);
        let ends = state.locations(LocationType::End).unwrap();
        assert_eq!(ends.len(), 1);
        assert!(ends.contains_key(&id));
    }

    #[test]
    fn remove_location_deletes_only_target_type() {
        let (mut state, id) = game_with_player();
        state.add_location(LocationType::Spawn, id, Point::new(1.0, 1.0));
        state.add_location(LocationType::End, id, Point::new(2.0, 2.0));

        state.remove_location(LocationType::Spawn, id);

        assert!(state.locations(LocationType::Spawn).unwrap().is_empty());
        // End 側は残る
        assert!(state
            .locations(LocationType::End)
            .unwrap()
            .contains_key(&id));
    }

    #[test]
    fn set_dragging_location_round_trips() {
        let (mut state, id) = game_with_player();
        assert!(state.dragging_location().is_none());

        state.set_dragging_location(Some(DraggingLocation {
            location_type: LocationType::Spawn,
            player_id: id,
        }));

        let dragging = state.dragging_location().expect("ドラッグ中");
        assert_eq!(dragging.location_type, LocationType::Spawn);
        assert_eq!(dragging.player_id, id);
    }
}
