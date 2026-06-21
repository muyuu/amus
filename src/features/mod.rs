pub mod debug_view;
pub mod eraser;
pub mod location;
pub mod main;
pub mod map;
pub mod player_info;
pub mod player_list;
pub mod route_drawing;
pub mod setup_dialog;
pub mod voice_memo;
pub mod welcome;
pub mod window;

#[cfg(not(target_arch = "wasm32"))]
use crate::resources::Resources;
#[cfg(not(target_arch = "wasm32"))]
use voice_memo::VoiceMemoFeature;

/// インスタンスを持つ Feature の集約
///
/// 状態を持つ Feature をまとめて管理する。
/// AmusApp から MainView に渡して使用する。
#[derive(Default)]
pub struct Features {
    #[cfg(not(target_arch = "wasm32"))]
    pub voice_memo: VoiceMemoFeature,
}

impl Features {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(resources: &Resources) -> Self {
        Self {
            voice_memo: VoiceMemoFeature::new(resources),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self {}
    }
}

