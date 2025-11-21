use crate::features::location::LocationInteraction;
use crate::features::route_drawing::RouteDrawingInteraction;
use crate::models::{User, Wave};
use crate::state::{AppState, DraggingLocation};

pub struct MainInteraction;

impl MainInteraction {
    /// すべてのインタラクション処理を実行
    pub fn handle_interactions(
        state: &mut AppState,
        users: &mut [User],
        wave: Option<&mut Wave>,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        let wave = match wave {
            Some(wave) => wave,
            None => return,
        };

        // 位置関連の処理（必要な値を先に取得）
        let dragging_location = state.dragging_location;
        Self::handle_location_interactions(
            users,
            wave,
            &mut state.dragging_location,
            dragging_location,
            response,
            ctx,
        );

        // ユーザードロップ処理（必要な値を先に取得）
        Self::handle_user_drop(wave, state.dragging_user_id, response, ctx);

        // エリアクリック処理（必要な値を先に取得）
        Self::handle_area_click(
            wave,
            state.dragging_location,
            state.selected_user_id,
            response,
        );

        Self::handle_route(wave, state.selected_user_id, response);

        // ドラッグ終了時のクリア
        Self::clear_drag_states(state, ctx);
    }

    /// 位置関連のインタラクション処理（ドラッグ中の位置更新とドラッグ開始検出）
    fn handle_location_interactions(
        users: &mut [User],
        wave: &mut Wave,
        dragging_location_state: &mut Option<DraggingLocation>,
        dragging_location: Option<DraggingLocation>,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        LocationInteraction::handle_end_location_click(users, wave, response);

        // 位置（出現位置・終了時位置）のドラッグ処理
        if let Some(dragging_location) = dragging_location {
            LocationInteraction::update_dragging_location(wave, dragging_location, response, ctx);
        }

        // ドラッグ開始の検出
        if response.drag_started() && dragging_location_state.is_none() {
            if let Some(location) = LocationInteraction::detect_drag_start(wave, response) {
                *dragging_location_state = Some(location);
            }
        }
    }

    /// ユーザードロップ処理
    fn handle_user_drop(
        wave: &mut Wave,
        dragging_user_id: Option<usize>,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        if let Some(dragging_user_id) = dragging_user_id {
            LocationInteraction::handle_user_drop(wave, dragging_user_id, response, ctx);
        }
    }

    /// エリアクリック処理（出現位置の設定）
    fn handle_area_click(
        wave: &mut Wave,
        dragging_location: Option<DraggingLocation>,
        selected_user_id: Option<usize>,
        response: &egui::Response,
    ) {
        if dragging_location.is_some() {
            return;
        }

        if let Some(selected_user_id) = selected_user_id {
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let point = LocationInteraction::screen_to_normalized_point(pos, response.rect);
                    LocationInteraction::set_spawn_location(wave, selected_user_id, point);
                }
            }
        }
    }

    fn handle_route(wave: &mut Wave, selected_user_id: Option<usize>, response: &egui::Response) {
        let user_id = match selected_user_id {
            Some(user_id) => user_id,
            None => return,
        };

        // マウス操作の処理（常にフリーハンド描画）
        RouteDrawingInteraction::handle_freehand_drawing(response, wave, user_id);
    }

    /// ドラッグ終了時の状態クリア
    fn clear_drag_states(state: &mut AppState, ctx: &egui::Context) {
        let has_dragging_user = state.dragging_user_id.is_some();
        let has_dragging_location = state.dragging_location.is_some();
        let pointer_released = ctx.input(|i| i.pointer.any_released());
        if has_dragging_user && pointer_released {
            state.dragging_user_id = None;
        }
        if has_dragging_location && pointer_released {
            state.dragging_location = None;
        }
    }
}
