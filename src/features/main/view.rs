use crate::app_action::AppAction;
use crate::common::aspect_ratio_centered;
use crate::features::debug_view::DebugFeature;
use crate::features::eraser::EraserFeature;
use crate::features::location::LocationFeature;
use crate::features::main::MainConstants;
use crate::features::map::MapFeature;
use crate::features::player_list::PlayerListFeature;
use crate::features::route_drawing::RouteDrawingFeature;
use crate::features::welcome::WelcomeFeature;
use crate::features::window::comms::CommsFeature;
use crate::features::window::lights::LightsFeature;
use crate::features::window::o2::O2Feature;
use crate::features::window::reactor::ReactorFeature;
use crate::features::window::turn::TurnFeature;
use crate::features::Features;
use crate::state::AppState;
use egui::*;

pub struct MainView;

impl MainView {
    pub fn render(state: &AppState, features: &mut Features, ui: &mut Ui) -> Vec<AppAction> {
        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let has_game = state.slices().game().has_game();
        if !has_game {
            return WelcomeFeature::render(ui, state);
        }

        // 全体の背景を灰色に
        let full_rect = ui.max_rect();
        ui.painter()
            .rect_filled(full_rect, 0.0, MainConstants::BG_COLOR);

        Self::render_main(state, features, ui)
    }

    fn render_main(state: &AppState, features: &mut Features, ui: &mut Ui) -> Vec<AppAction> {
        let mut actions = aspect_ratio_centered(ui, MainConstants::VIEW_RATIO, |ui| {
            // メイン領域の描画領域レスポンスを取得
            let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

            let mut acts = Vec::new();
            MapFeature::render(state, &response, ui);
            acts.extend(PlayerListFeature::render(state, ui));
            acts.extend(RouteDrawingFeature::render(state, &response, ui));
            DebugFeature::render(state, ui.ctx());
            acts.extend(LocationFeature::render(state, &response, ui));
            acts.extend(EraserFeature::render(state, ui));
            acts
        });

        actions.extend(Self::render_windows(state, features, ui));
        actions
    }

    // 各種機能ウィンドウの描画
    #[allow(unused_variables)]
    fn render_windows(state: &AppState, features: &mut Features, ui: &mut Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        actions.extend(TurnFeature::render(state, ui));
        actions.extend(CommsFeature::render(state, ui));
        actions.extend(LightsFeature::render(state, ui));
        actions.extend(O2Feature::render(state, ui));
        actions.extend(ReactorFeature::render(state, ui));

        #[cfg(not(target_arch = "wasm32"))]
        actions.extend(features.voice_memo.render(state, ui));

        actions
    }
}
