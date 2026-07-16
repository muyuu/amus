mod eraser;
mod game;
mod location;
mod player;
mod player_info;
mod route_drawing;
mod setup;
mod wave;

pub use eraser::EraserAction;
pub use game::GameAction;
pub use location::LocationAction;
pub use player::PlayerAction;
pub use player_info::PlayerInfoAction;
pub use route_drawing::RouteDrawingAction;
pub use setup::SetupAction;
pub use wave::WaveAction;

use super::AppState;

/// 状態変更を集約する構造体
///
/// ViewはAppStateを直接変更せず、Actionを返す。
/// Actionsがそれを受けて状態を更新する。
///
/// 各ドメインのハンドラーは別ファイルで impl される:
/// - player.rs: handle_player()
/// - game.rs: handle_game()
/// - location.rs: handle_location()
pub struct Actions<'a> {
    pub(super) state: &'a mut AppState,
}

impl<'a> Actions<'a> {
    pub fn new(state: &'a mut AppState) -> Self {
        Self { state }
    }
}
