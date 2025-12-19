mod eraser;
mod game;
mod location;
mod player;
mod route_drawing;

pub use eraser::EraserAction;
pub use game::GameAction;
pub use location::LocationAction;
pub use player::PlayerAction;
pub use route_drawing::RouteDrawingAction;

use super::AppState;

/// 状態変更を集約する構造体
///
/// ViewはAppStateを直接変更せず、Actionを返す。
/// Actionsがそれを受けて状態を更新する。
///
/// 各ドメインのハンドラーは別ファイルで impl される:
/// - player.rs: handle_player()
/// - game.rs: handle_game() (将来)
/// - location.rs: handle_location() (将来)
pub struct Actions<'a> {
    pub(super) state: &'a AppState,
}

impl<'a> Actions<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}
