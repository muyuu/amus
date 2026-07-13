use serde::{Deserialize, Serialize};

use crate::constants::GridIds;
use crate::i18n::keys::{self, TextKey};

/// サボタージュ種別。種別ごとに異なる表示情報（タイトル・グリッド ID・パネル位置）を
/// この 1 箇所に集約し、種別追加時の横断的な複製をなくす。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Sabotage {
    Comms,
    Lights,
    O2,
    Reactor,
}

impl Sabotage {
    /// 全種別を表示順に並べた配列。
    pub const ALL: [Sabotage; 4] = [
        Sabotage::Comms,
        Sabotage::Lights,
        Sabotage::O2,
        Sabotage::Reactor,
    ];

    /// ウィンドウタイトルに使う翻訳キー。
    pub fn title_key(self) -> TextKey {
        match self {
            Sabotage::Comms => keys::SABOTAGE_COMMS,
            Sabotage::Lights => keys::SABOTAGE_LIGHTS,
            Sabotage::O2 => keys::SABOTAGE_O2,
            Sabotage::Reactor => keys::SABOTAGE_REACTOR,
        }
    }

    /// egui グリッドの ID。
    pub fn grid_id(self) -> &'static str {
        match self {
            Sabotage::Comms => GridIds::COMMS,
            Sabotage::Lights => GridIds::LIGHTS,
            Sabotage::O2 => GridIds::O2,
            Sabotage::Reactor => GridIds::REACTOR,
        }
    }

    /// パネルの初期表示位置の X オフセット。種別ごとに横に並べて重なりを避ける。
    pub fn panel_offset_x(self) -> f32 {
        match self {
            Sabotage::Comms => 100.0,
            Sabotage::Lights => 200.0,
            Sabotage::O2 => 300.0,
            Sabotage::Reactor => 400.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_covers_every_variant_in_display_order() {
        assert_eq!(
            Sabotage::ALL,
            [
                Sabotage::Comms,
                Sabotage::Lights,
                Sabotage::O2,
                Sabotage::Reactor
            ]
        );
    }

    #[test]
    fn title_key_maps_to_matching_i18n_key() {
        assert_eq!(Sabotage::Comms.title_key(), keys::SABOTAGE_COMMS);
        assert_eq!(Sabotage::Lights.title_key(), keys::SABOTAGE_LIGHTS);
        assert_eq!(Sabotage::O2.title_key(), keys::SABOTAGE_O2);
        assert_eq!(Sabotage::Reactor.title_key(), keys::SABOTAGE_REACTOR);
    }

    #[test]
    fn grid_id_maps_to_matching_grid() {
        assert_eq!(Sabotage::Comms.grid_id(), GridIds::COMMS);
        assert_eq!(Sabotage::Lights.grid_id(), GridIds::LIGHTS);
        assert_eq!(Sabotage::O2.grid_id(), GridIds::O2);
        assert_eq!(Sabotage::Reactor.grid_id(), GridIds::REACTOR);
    }
}
