use crate::common::CommonTexts;
use crate::features::debug_view::DebugView;
use crate::features::main::MainInteraction;
use crate::features::map::MapView;
use crate::features::route_drawing::RouteDrawingInteraction;
use crate::features::welcome::WelcomeView;
use crate::models::Game;
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

        let area_name = game.area.name.clone();

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

    /// マップ描画
    fn draw_map(
        game: &Game,
        current_wave_index: usize,
        selected_user_id: Option<usize>,
        asset_manager: Option<&crate::assets::AssetManager>,
        response: &egui::Response,
        ui: &mut egui::Ui,
    ) {
        let wave = match game.get_wave(current_wave_index) {
            Some(wave) => wave,
            None => return,
        };

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
