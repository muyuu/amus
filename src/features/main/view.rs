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
use crate::state::Slices;
use egui::*;

pub struct MainView;

impl MainView {
    pub fn render(slices: &Slices, features: &mut Features, ui: &mut Ui) -> Vec<AppAction> {
        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        if !slices.game().has_game() {
            return WelcomeFeature::render(slices, ui);
        }

        // 全体の背景を灰色に
        let full_rect = ui.max_rect();
        ui.painter()
            .rect_filled(full_rect, 0.0, MainConstants::BG_COLOR);

        Self::render_main(slices, features, ui)
    }

    fn render_main(slices: &Slices, features: &mut Features, ui: &mut Ui) -> Vec<AppAction> {
        let mut actions = aspect_ratio_centered(ui, MainConstants::VIEW_RATIO, |ui| {
            // メイン領域の描画領域レスポンスを取得
            let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

            let mut acts = Vec::new();
            acts.extend(MapFeature::render(slices, &response, ui));
            acts.extend(PlayerListFeature::render(slices, ui));
            acts.extend(RouteDrawingFeature::render(slices, &response, ui));
            acts.extend(DebugFeature::render(slices, ui));
            acts.extend(LocationFeature::render(slices, &response, ui));
            acts.extend(EraserFeature::render(slices, ui));
            acts
        });

        actions.extend(Self::render_windows(slices, features, ui));
        actions
    }

    // 各種機能ウィンドウの描画
    #[allow(unused_variables)]
    fn render_windows(slices: &Slices, features: &mut Features, ui: &mut Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        actions.extend(TurnFeature::render(slices, ui));
        actions.extend(CommsFeature::render(slices, ui));
        actions.extend(LightsFeature::render(slices, ui));
        actions.extend(O2Feature::render(slices, ui));
        actions.extend(ReactorFeature::render(slices, ui));

        #[cfg(not(target_arch = "wasm32"))]
        actions.extend(features.voice_memo.render(slices, ui));

        actions
    }
}
