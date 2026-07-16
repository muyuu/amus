//! アプリ全体のアクション。
//!
//! 各 Feature の View は `Vec<AppAction>` を返すだけにし、`AmusApp::handle_actions` が
//! 1 箇所で dispatch する（中央 dispatch）。詳細は docs/decisions.md 0001 を参照。

#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
use crate::features::voice_memo::VoiceMemoAction;
use crate::state::{
    EraserAction, GameAction, LocationAction, PlayerAction, PlayerInfoAction, RouteDrawingAction,
    SetupAction, WaveAction,
};

/// 各ドメインの Action を束ねるアプリ全体のアクション。
pub enum AppAction {
    Game(GameAction),
    Player(PlayerAction),
    PlayerInfo(PlayerInfoAction),
    Location(LocationAction),
    RouteDrawing(RouteDrawingAction),
    Eraser(EraserAction),
    Setup(SetupAction),
    Wave(WaveAction),
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    VoiceMemo(VoiceMemoAction),
}
