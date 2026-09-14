pub mod debug_view;
pub mod eraser;
pub mod location;
pub mod main;
pub mod map;
pub mod player_info;
pub mod player_list;
pub mod route_drawing;
pub mod settings;
pub mod setup_dialog;
// voice_memo は録音・Whisper 書き起こし（cpal / whisper-rs）を使うネイティブ専用機能。
// 重い native 依存を切り離せるよう voice_memo feature（default on）で opt-out 可能。
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod voice_memo;
pub mod welcome;
pub mod window;

#[cfg(not(target_arch = "wasm32"))]
use crate::resources::Resources;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
use voice_memo::VoiceMemoFeature;

/// 状態（インスタンス）を持つ Feature の集約。
///
/// stateless な Feature は unit struct（`XxxFeature`）で `render` を静的に呼ぶだけだが、
/// フレームをまたいで状態を保持する Feature はインスタンスをここに集約し、`AmusApp` が
/// 所有して `MainView` に渡す。
#[derive(Default)]
pub struct StatefulFeatures {
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub voice_memo: VoiceMemoFeature,
}

impl StatefulFeatures {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(_resources: &Resources) -> Self {
        Self {
            #[cfg(feature = "voice_memo")]
            voice_memo: VoiceMemoFeature::new(_resources),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self {}
    }
}
