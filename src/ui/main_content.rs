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

        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let game = match &mut state.game {
            Some(game) => game,
            None => {
                WelcomeView::show(ui, state);
                return;
            }
        };

        // 先に必要な情報を取得
        let area_name = game.area.name.clone();
        let current_wave_index = state.current_wave_index;

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // 位置（出現位置・終了時位置）のドラッグ処理（先に処理）
        let dragging_location_value = state.dragging_location;
        if let Some(dragging_location) = dragging_location_value {
            LocationInteraction::update_dragging_location(
                game,
                current_wave_index,
                dragging_location,
                &response,
                ui.ctx(),
            );
        }

        // ドラッグ開始の検出（gameの不変参照を使用）
        if response.drag_started() {
            let dragging_location = state.dragging_location;
            if dragging_location.is_none() {
                // gameの不変参照を取得（gameの可変借用を一時的に解放）
                let game_ref: &Game = &*game;
                let location =
                    LocationInteraction::detect_drag_start(game_ref, current_wave_index, &response);
                if let Some(location) = location {
                    state.dragging_location = Some(location);
                }
            }
        }

        // ドラッグ&ドロップの処理
        let dragging_user_id = state.dragging_user_id;
        if let Some(dragging_user_id) = dragging_user_id {
            // ユーザーをドロップした位置に終了時位置を設定
            LocationInteraction::handle_user_drop(
                game,
                current_wave_index,
                dragging_user_id,
                &response,
                ui.ctx(),
            );
        }

        // エリアのクリック処理（選択中のユーザーがいる場合）
        // 位置ドラッグ中でない場合のみ処理
        if state.dragging_location.is_none() {
            let selected_user_id = state.selected_user_id;
            if let Some(selected_user_id) = selected_user_id {
                // クリックで出現位置を設定
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let point =
                            LocationInteraction::screen_to_normalized_point(pos, response.rect);
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

        // ドラッグ終了時にクリア（マウスボタンが離された時）
        if state.dragging_user_id.is_some() && ui.ctx().input(|i| i.pointer.any_released()) {
            state.dragging_user_id = None;
        }
        if state.dragging_location.is_some() && ui.ctx().input(|i| i.pointer.any_released()) {
            state.dragging_location = None;
        }

        // マップ描画（不変参照で描画）
        // 必要な情報を先に取得
        let selected_user_id = state.selected_user_id;
        let show_debug_view = state.show_debug_view;
        let dragging_user_id = state.dragging_user_id;

        if let Some(wave) = game.get_wave(current_wave_index) {
            let painter = ui.painter_at(response.rect);
            MapView::draw(
                &painter,
                &response,
                game,
                wave,
                selected_user_id,
                state.asset_manager.as_ref(),
            );
        }

        // 可変参照を取得して編集
        if let Some(wave) = game.get_wave_mut(current_wave_index) {
            ui.heading(format!(
                "{} {} - {}",
                common.label_turn_prefix,
                current_wave_index + 1,
                area_name
            ));

            // マウス操作の処理（常にフリーハンド描画）
            if let Some(user_id) = state.selected_user_id {
                RouteDrawingInteraction::handle_freehand_drawing(&response, wave, user_id);
            }
        } // waveの可変借用を解放

        // デバッグビューの表示（stateの可変借用を解放した後）
        if show_debug_view {
            let selected_user_id = state.selected_user_id;
            DebugView::show(ui.ctx(), dragging_user_id, selected_user_id, game);
        }
    }
}
