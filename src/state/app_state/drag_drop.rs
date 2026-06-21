use std::collections::HashMap;

use crate::models::{DraggingLocation, LocationType, PlayerId, Point};

use super::AppState;

// ドラッグ&ドロップ関連
impl AppState {
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
