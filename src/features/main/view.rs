use crate::common::CommonTexts;
use crate::features::debug_view::DebugView;
use crate::features::main::MainInteraction;
use crate::features::map::MapView;
use crate::features::route_drawing::{RouteDrawingInteraction, RouteDrawingView};
use crate::features::welcome::WelcomeView;
use crate::models::{Game, Wave};
use crate::state::AppState;

pub struct MainView;

impl MainView {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        // 先に必要な情報を取得（借用チェッカーの問題を回避）
        let common = CommonTexts::get(state);
        let current_wave_index = state.current_wave_index;

        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let has_game = state.game.is_some();
        if !has_game {
            WelcomeView::show(ui, state);
            return;
        }

        if state.show_debug_view {
            DebugView::show(
                ui.ctx(),
                state.dragging_user_id,
                state.selected_user_id,
                state.game.as_ref().unwrap(),
            );
        }

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // インタラクション処理（別モジュールに分離）
        // gameの可変借用を一時的に解放してから呼び出し
        {
            // state.gameの可変借用を一時的に解放するため、Noneに置き換えてから再度取得
            let game_opt = state.game.take();
            if let Some(mut game) = game_opt {
                MainInteraction::handle_interactions(
                    state,
                    &mut game,
                    current_wave_index,
                    &response,
                    ui.ctx(),
                );
                state.game = Some(game);
            }
        }

        // gameを再度取得
        let game = match &mut state.game {
            Some(game) => game,
            None => return,
        };

        let wave = match game.get_wave(current_wave_index) {
            Some(wave) => wave,
            None => return,
        };

        let area_name = game.area.name();

        // マップ描画（必要な値を先に取得）
        Self::draw_map(
            game,
            &wave,
            state.selected_user_id,
            state.asset_manager.as_ref(),
            &response,
            ui,
        );

        Self::render_route(
            &game,
            &wave,
            &response,
            ui,
        );

        // UI表示とフリーハンド描画（必要な値を先に取得）
        Self::show_ui_content(
            game,
            current_wave_index,
            state.selected_user_id,
            &common,
            &area_name,
            &response,
            ui,
        );
    }

    /// マップ描画
    fn draw_map(
        game: &Game,
        wave: &Wave,
        selected_user_id: Option<usize>,
        asset_manager: Option<&crate::assets::AssetManager>,
        response: &egui::Response,
        ui: &mut egui::Ui,
    ) {
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

    /// 手書きルートの描画
    fn render_route(
        game: &Game,
        wave: &Wave,
        response: &egui::Response,
        ui: &mut egui::Ui,
    ) {
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        // 既存のルートを描画
        for route in &wave.routes {
            if let Some(user) = game.users.get(route.user_id) {
                let color = user.color.to_egui_color();
                let spawn_location = wave.spawn_locations.get(&route.user_id);
                RouteDrawingView::draw_route(&painter, route, spawn_location, color, rect);
            }
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
        let wave = match game.get_wave_mut(current_wave_index) {
            Some(wave) => wave,
            None => return,
        };

        ui.heading(format!(
            "{} {} - {}",
            common.label_turn_prefix,
            current_wave_index + 1,
            area_name
        ));

        let user_id = match selected_user_id {
            Some(user_id) => user_id,
            None => return,
        };

        // マウス操作の処理（常にフリーハンド描画）
        RouteDrawingInteraction::handle_freehand_drawing(response, wave, user_id);
    }
}
