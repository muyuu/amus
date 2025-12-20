use std::collections::HashMap;

use crate::models::location::LocationType;
use crate::models::player::PlayerId;
use crate::models::{Point, Route, Wave};
use crate::state::app_data::AppData;

/// ウェーブ関連の読み取り専用アクセスを提供
pub struct WaveSlice<'a> {
    data: &'a AppData,
}

impl<'a> WaveSlice<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// 現在のウェーブインデックス
    pub fn current_wave_index(&self) -> usize {
        self.data.current_wave_index
    }

    /// 現在のウェーブへの参照を取得
    pub fn current_wave(&self) -> Option<&Wave> {
        let game = self.data.game.as_ref()?;
        game.get_wave(self.data.current_wave_index)
    }

    /// 現在のウェーブのルート一覧
    pub fn routes(&self) -> Option<&[Route]> {
        self.current_wave().map(|w| w.routes.as_slice())
    }

    /// 指定タイプの位置情報を取得
    pub fn locations(&self, location_type: LocationType) -> Option<&HashMap<PlayerId, Point>> {
        let wave = self.current_wave()?;
        match location_type {
            LocationType::Spawn => Some(&wave.spawn_locations),
            LocationType::End => Some(&wave.end_locations),
        }
    }

    /// 出現位置の一覧
    pub fn spawn_locations(&self) -> Option<&HashMap<PlayerId, Point>> {
        self.locations(LocationType::Spawn)
    }

    /// 終了時位置の一覧
    pub fn end_locations(&self) -> Option<&HashMap<PlayerId, Point>> {
        self.locations(LocationType::End)
    }

    /// 全ウェーブ数
    pub fn wave_count(&self) -> usize {
        self.data
            .game
            .as_ref()
            .map(|g| g.waves.len())
            .unwrap_or(0)
    }
}
