use crate::features::debug_view::DebugView;
use crate::features::location::LocationFeature;
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

        MapView::render(state, &response, ui);
        LocationFeature::render(state, &response, ui);
        RouteDrawingFeature::render(state, &response, ui);
        DebugView::render(state, state.show_debug_view(), ui.ctx());
    }
}
