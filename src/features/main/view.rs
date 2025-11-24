use crate::features::debug_view::DebugView;
use crate::features::location::LocationView;
use crate::features::main::MainInteraction;
use crate::features::map::MapView;
use crate::features::route_drawing::RouteDrawingFeature;
use crate::features::welcome::WelcomeView;
use crate::state::AppState;

pub struct MainView;

impl MainView {
    pub fn render(state: &mut AppState, ui: &mut egui::Ui) {
        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let has_game = state.game().is_some();
        if !has_game {
            WelcomeView::render(ui, state);
            return;
        }

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        Self::render_children(state, &response, ui);
        Self::setup_interactions(state, &response, ui);
    }

    fn render_children(state: &mut AppState, response: &egui::Response, ui: &mut egui::Ui) {
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        MapView::render(state, &painter, response);
        LocationView::render(state, &painter, rect);
        RouteDrawingFeature::render(state, response, ui);
        DebugView::render(state, state.show_debug_view(), ui.ctx());
    }

    fn setup_interactions(state: &mut AppState, response: &egui::Response, ui: &mut egui::Ui) {
        MainInteraction::handle_interactions(state, response, ui.ctx());
    }
}
