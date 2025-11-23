use crate::features::debug_view::DebugView;
use crate::features::location::LocationView;
use crate::features::main::MainInteraction;
use crate::features::map::MapView;
use crate::features::route_drawing::RouteDrawingView;
use crate::features::welcome::WelcomeView;
use crate::models::{Game, Wave};
use crate::state::AppState;

pub struct MainView;

impl MainView {
    pub fn render(state: &mut AppState, ui: &mut egui::Ui) {
        // 先に必要な情報を取得（借用チェッカーの問題を回避）
        let current_wave_index = state.current_wave_index().clone();

        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let has_game = state.game().is_some();
        if !has_game {
            WelcomeView::render(ui, state);
            return;
        }

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // gameを再度取得
        let game = match state.game() {
            Some(game) => game,
            None => return,
        };

        let wave = match game.get_wave(current_wave_index) {
            Some(wave) => wave,
            None => return,
        };

        Self::render_children(state, &game, wave, &response, ui);
        Self::setup_interactions(state, &response, ui);
    }

    fn render_children(
        state: &AppState,
        game: &Game,
        wave: &Wave,
        response: &egui::Response,
        ui: &mut egui::Ui,
    ) {
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;
        let selected_user_id = state.selected_user_id();
        let asset_manager = state.asset_manager();
        let dragging_user_id = state.dragging_user_id();

        MapView::render(
            &game.area,
            &painter,
            response,
            selected_user_id,
            asset_manager,
        );
        LocationView::render(&painter, game, wave, rect);
        RouteDrawingView::render(game, wave, &painter, rect);
        DebugView::render(
            state.show_debug_view(),
            ui.ctx(),
            dragging_user_id,
            selected_user_id,
            game,
        );
    }

    fn setup_interactions(state: &mut AppState, response: &egui::Response, ui: &mut egui::Ui) {
        MainInteraction::handle_interactions(state, response, ui.ctx());
    }
}
