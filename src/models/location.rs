use crate::models::player::PlayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    Spawn, // 出現位置
    End,   // 終了時位置
}

#[derive(Debug, Clone, Copy)]
pub struct DraggingLocation {
    pub location_type: LocationType,
    pub player_id: PlayerId,
}
