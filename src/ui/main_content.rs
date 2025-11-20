use crate::common::CommonTexts;
use crate::features::debug_view::DebugView;
use crate::features::location::LocationInteraction;
use crate::features::map::MapView;
use crate::features::route_drawing::RouteDrawingInteraction;
use crate::features::welcome::WelcomeView;
use crate::models::*;
use crate::state::AppState;

pub struct MainContent;

impl MainContent {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        // 先に必要な情報を取得（借用チェッカーの問題を回避）
        let common = CommonTexts::get(state);
        let current_wave_index = state.current_wave_index;

        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let game = match &mut state.game {
            Some(game) => game,
            None => {
                WelcomeView::show(ui, state);
                return;
            }
        };

        let area_name = game.area.name.clone();

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // 位置関連の処理（必要な値を先に取得）
        let dragging_location = state.dragging_location;
        Self::handle_location_interactions(
            &mut state.dragging_location,
            game,
            current_wave_index,
            dragging_location,
            &response,
            ui.ctx(),
        );

        // ユーザードロップ処理（必要な値を先に取得）
        let dragging_user_id = state.dragging_user_id;
        Self::handle_user_drop(
            game,
            current_wave_index,
            dragging_user_id,
            &response,
            ui.ctx(),
        );

        // エリアクリック処理（必要な値を先に取得）
        let dragging_location_for_click = state.dragging_location;
        let selected_user_id_for_click = state.selected_user_id;
        Self::handle_area_click(
            game,
            current_wave_index,
            dragging_location_for_click,
            selected_user_id_for_click,
            &response,
        );

        // ドラッグ終了時のクリア（必要な値を先に取得）
        let has_dragging_user = state.dragging_user_id.is_some();
        let has_dragging_location = state.dragging_location.is_some();
        let pointer_released = ui.ctx().input(|i| i.pointer.any_released());
        if has_dragging_user && pointer_released {
            state.dragging_user_id = None;
        }
        if has_dragging_location && pointer_released {
            state.dragging_location = None;
        }

        // マップ描画（必要な値を先に取得）
        let selected_user_id = state.selected_user_id;
        let show_debug_view = state.show_debug_view;
        let dragging_user_id = state.dragging_user_id;
        let asset_manager = state.asset_manager.as_ref();
        Self::draw_map(
            game,
            current_wave_index,
            selected_user_id,
            asset_manager,
            &response,
            ui,
        );

        // UI表示とフリーハンド描画（必要な値を先に取得）
        let selected_user_id_for_ui = state.selected_user_id;
        Self::show_ui_content(
            game,
            current_wave_index,
            selected_user_id_for_ui,
            &common,
            &area_name,
            &response,
            ui,
        );

        // デバッグビューの表示
        Self::show_debug_view_if_needed(
            show_debug_view,
            dragging_user_id,
            selected_user_id,
            game,
            ui.ctx(),
        );
    }

    /// 位置関連のインタラクション処理（ドラッグ中の位置更新とドラッグ開始検出）
    fn handle_location_interactions(
        dragging_location_state: &mut Option<crate::state::DraggingLocation>,
        game: &mut Game,
        current_wave_index: usize,
        dragging_location: Option<crate::state::DraggingLocation>,
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
        dragging_location: Option<crate::state::DraggingLocation>,
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

    /// マップ描画
    fn draw_map(
        game: &Game,
        current_wave_index: usize,
        selected_user_id: Option<usize>,
        asset_manager: Option<&crate::assets::AssetManager>,
        response: &egui::Response,
        ui: &mut egui::Ui,
    ) {
        if let Some(wave) = game.get_wave(current_wave_index) {
            let painter = ui.painter_at(response.rect);
            MapView::draw(
                &painter,
                response,
                game,
                wave,
                selected_user_id,
                asset_manager,
            );
        }
    }

    /// UI表示とフリーハンド描画
    fn show_ui_content(
        game: &mut Game,
        current_wave_index: usize,
        selected_user_id: Option<usize>,
        common: &CommonTexts,
        area_name: &str,
        response: &egui::Response,
        ui: &mut egui::Ui,
    ) {
        if let Some(wave) = game.get_wave_mut(current_wave_index) {
            ui.heading(format!(
                "{} {} - {}",
                common.label_turn_prefix,
                current_wave_index + 1,
                area_name
            ));

            // マウス操作の処理（常にフリーハンド描画）
            if let Some(user_id) = selected_user_id {
                RouteDrawingInteraction::handle_freehand_drawing(response, wave, user_id);
            }
        }
    }

    /// デバッグビューの表示
    fn show_debug_view_if_needed(
        show_debug_view: bool,
        dragging_user_id: Option<usize>,
        selected_user_id: Option<usize>,
        game: &Game,
        ctx: &egui::Context,
    ) {
        if show_debug_view {
            DebugView::show(ctx, dragging_user_id, selected_user_id, game);
        }
    }
}
