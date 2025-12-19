use crate::common::aspect_ratio_centered;
use crate::features::debug_view::DebugView;
use crate::features::eraser::EraserFeature;
use crate::features::location::LocationFeature;
use crate::features::main::MainConstants;
use crate::features::map::MapFeature;
use crate::features::player_list::PlayerListFeature;
use crate::features::route_drawing::RouteDrawingFeature;
use crate::features::welcome::WelcomeView;
use crate::features::window::comms::CommsFeature;
use crate::features::window::lights::LightsFeature;
use crate::features::window::o2::O2Feature;
use crate::features::window::reactor::ReactorFeature;
use crate::features::window::turn::TurnFeature;
use crate::state::AppState;
use egui::*;

pub struct MainView;

impl MainView {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let has_game = state.game().is_some();
        if !has_game {
            WelcomeView::render(ui, state);
            return;
        }

        // 全体の背景を灰色に
        let full_rect = ui.max_rect();
        ui.painter()
            .rect_filled(full_rect, 0.0, MainConstants::BG_COLOR);

        Self::render_main(state, ui);
    }

    fn render_main(state: &mut AppState, ui: &mut Ui) {
        aspect_ratio_centered(ui, MainConstants::VIEW_RATIO, |ui| {
            // メイン領域の描画領域レスポンスを取得
            let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

            MapFeature::render(state, &response, ui);
            PlayerListFeature::render(state, ui);
            RouteDrawingFeature::render(state, &response, ui);
            DebugView::render(state, state.show_debug_view(), ui.ctx());
            LocationFeature::render(state, &response, ui);
            EraserFeature::render(state, ui);
        });

        Self::render_windows(state, ui);
    }

    // 各種機能ウィンドウの描画
    fn render_windows(state: &mut AppState, ui: &mut Ui) {
        TurnFeature::render(state, ui);
        CommsFeature::render(state, ui);
        LightsFeature::render(state, ui);
        O2Feature::render(state, ui);
        ReactorFeature::render(state, ui);
    }
}
