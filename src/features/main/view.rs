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
        let current_wave_index = state.current_wave_index;

        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let has_game = state.game.is_some();
        if !has_game {
            WelcomeView::render(ui, state);
            return;
        }

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // gameを再度取得
        let game = match &mut state.game {
            Some(game) => game,
            None => return,
        };

        let wave = match game.get_wave(current_wave_index) {
            Some(wave) => wave,
            None => return,
        };

        let painter = ui.painter_at(response.rect);
        let rect = response.rect;
        MapView::render(
            &game.area,
            &painter,
            &response,
            state.selected_user_id,
            state.asset_manager.as_ref(),
        );
        LocationView::render(&painter, game, wave, rect);
        Self::render_route(game, wave, &painter, rect);
        DebugView::render(
            state.show_debug_view,
            ui.ctx(),
            state.dragging_user_id,
            state.selected_user_id,
            state.game.as_ref().unwrap(),
        );

        // インタラクション処理（別モジュールに分離）
        // gameの可変借用を一時的に解放してから呼び出し
        {
            // state.gameの可変借用を一時的に解放するため、Noneに置き換えてから再度取得
            let game_opt = state.game.take();
            if let Some(mut game) = game_opt {
                let wave = match game.get_wave_mut(current_wave_index) {
                    Some(wave) => wave,
                    None => return,
                };
                MainInteraction::handle_interactions(state, wave, &response, ui.ctx());
                state.game = Some(game);
            }
        }
    }

    /// 手書きルートの描画
    fn render_route(game: &Game, wave: &Wave, painter: &egui::Painter, rect: egui::Rect) {
        // 既存のルートを描画
        for route in &wave.routes {
            if let Some(user) = game.users.get(route.user_id) {
                let color = user.color.to_egui_color();
                let spawn_location = wave.spawn_locations.get(&route.user_id);
                RouteDrawingView::draw_route(painter, route, spawn_location, color, rect);
            }
        }
    }
}
