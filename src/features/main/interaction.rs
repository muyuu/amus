use crate::features::location::LocationInteraction;
use crate::features::route_drawing::RouteDrawingInteraction;
use crate::state::AppState;

pub struct MainInteraction;

impl MainInteraction {
    /// すべてのインタラクション処理を実行
    pub fn handle_interactions(
        state: &mut AppState,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        // 位置関連の処理（必要な値を先に取得）
        Self::handle_location_interactions(state, response, ctx);

        // ユーザードロップ処理（必要な値を先に取得）
        Self::handle_user_drop(state, response, ctx);

        // エリアクリック処理（必要な値を先に取得）
        Self::handle_area_click(state, response);

        Self::handle_route(state, response);

        // ドラッグ終了時のクリア
        Self::clear_drag_states(state, ctx);
    }

    /// 位置関連のインタラクション処理（ドラッグ中の位置更新とドラッグ開始検出）
    fn handle_location_interactions(
        state: &mut AppState,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        LocationInteraction::handle_end_location_click(state, response);

        // 位置（出現位置・終了時位置）のドラッグ処理
        if let Some(dragging_location) = state.dragging_location() {
            LocationInteraction::update_dragging_location(state, dragging_location, response, ctx);
        }

        // ドラッグ開始の検出
        if response.drag_started() && state.dragging_location().is_none() {
            if let Some(location) = LocationInteraction::detect_drag_start(state, response) {
                state.set_dragging_location(Some(location));
            }
        }
    }

    /// ユーザードロップ処理
    fn handle_user_drop(state: &mut AppState, response: &egui::Response, ctx: &egui::Context) {
        LocationInteraction::handle_user_drop(state, response, ctx);
    }

    /// エリアクリック処理（出現位置の設定）
    fn handle_area_click(state: &mut AppState, response: &egui::Response) {
        if state.dragging_location().is_some() {
            return;
        }

        if let Some(selected_user_id) = state.selected_user_id() {
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let point = LocationInteraction::screen_to_normalized_point(pos, response.rect);
                    LocationInteraction::set_spawn_location(state, selected_user_id, point);
                }
            }
        }
    }

    fn handle_route(state: &mut AppState, response: &egui::Response) {
        let user_id = match state.selected_user_id() {
            Some(user_id) => user_id,
            None => return,
        };

        // マウス操作の処理（常にフリーハンド描画）
        RouteDrawingInteraction::handle_freehand_drawing(state, response, user_id);
    }

    /// ドラッグ終了時の状態クリア
    fn clear_drag_states(state: &mut AppState, ctx: &egui::Context) {
        let has_dragging_user = state.dragging_user_id().is_some();
        let has_dragging_location = state.dragging_location().is_some();
        let pointer_released = ctx.input(|i| i.pointer.any_released());

        if has_dragging_user && pointer_released {
            state.set_dragging_user_id(None);
        }
        if has_dragging_location && pointer_released {
            state.set_dragging_location(None);
        }
    }
}
