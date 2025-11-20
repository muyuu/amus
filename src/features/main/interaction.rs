use crate::features::location::LocationInteraction;
use crate::models::Game;
use crate::state::{AppState, DraggingLocation};

pub struct MainInteraction;

impl MainInteraction {
    /// すべてのインタラクション処理を実行
    pub fn handle_interactions(
        state: &mut AppState,
        game: &mut Game,
        current_wave_index: usize,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        // 位置関連の処理（必要な値を先に取得）
        let dragging_location = state.dragging_location;
        Self::handle_location_interactions(
            &mut state.dragging_location,
            game,
            current_wave_index,
            dragging_location,
            response,
            ctx,
        );

        // ユーザードロップ処理（必要な値を先に取得）
        let dragging_user_id = state.dragging_user_id;
        Self::handle_user_drop(game, current_wave_index, dragging_user_id, response, ctx);

        // エリアクリック処理（必要な値を先に取得）
        let dragging_location_for_click = state.dragging_location;
        let selected_user_id = state.selected_user_id;
        Self::handle_area_click(
            game,
            current_wave_index,
            dragging_location_for_click,
            selected_user_id,
            response,
        );

        // ドラッグ終了時のクリア
        Self::clear_drag_states(state, ctx);
    }

    /// 位置関連のインタラクション処理（ドラッグ中の位置更新とドラッグ開始検出）
    fn handle_location_interactions(
        dragging_location_state: &mut Option<DraggingLocation>,
        game: &mut Game,
        current_wave_index: usize,
        dragging_location: Option<DraggingLocation>,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        // 位置（出現位置・終了時位置）のドラッグ処理
        if let Some(dragging_location) = dragging_location {
            LocationInteraction::update_dragging_location(
                game,
                current_wave_index,
                dragging_location,
                response,
                ctx,
            );
        }

        // ドラッグ開始の検出
        if response.drag_started() && dragging_location_state.is_none() {
            let game_ref: &Game = &*game;
            if let Some(location) =
                LocationInteraction::detect_drag_start(game_ref, current_wave_index, response)
            {
                *dragging_location_state = Some(location);
            }
        }
    }

    /// ユーザードロップ処理
    fn handle_user_drop(
        game: &mut Game,
        current_wave_index: usize,
        dragging_user_id: Option<usize>,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        if let Some(dragging_user_id) = dragging_user_id {
            LocationInteraction::handle_user_drop(
                game,
                current_wave_index,
                dragging_user_id,
                response,
                ctx,
            );
        }
    }

    /// エリアクリック処理（出現位置の設定）
    fn handle_area_click(
        game: &mut Game,
        current_wave_index: usize,
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
                    LocationInteraction::set_spawn_location(
                        game,
                        current_wave_index,
                        selected_user_id,
                        point,
                    );
                }
            }
        }
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
