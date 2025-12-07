use crate::common::aspect_ratio_centered;
use crate::features::comms::CommsFeature;
use crate::features::debug_view::DebugView;
use crate::features::eraser::EraserFeature;
use crate::features::lights::LightsFeature;
use crate::features::location::LocationFeature;
use crate::features::main::MainConstants;
use crate::features::map::MapView;
use crate::features::route_drawing::RouteDrawingFeature;
use crate::features::turn::TurnFeature;
use crate::features::welcome::WelcomeView;
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
        ui.painter().rect_filled(
            full_rect,
            0.0,
            Color32::from_rgb(128, 128, 128), // 灰色
        );

        aspect_ratio_centered(ui, MainConstants::VIEW_RATIO, |ui| {
            // マップ表示エリア（ここにエリア画像と軌跡を描画）
            let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

            MapView::render(state, &response, ui);
            RouteDrawingFeature::render(state, &response, ui);
            LocationFeature::render(state, &response, ui);
            DebugView::render(state, state.show_debug_view(), ui.ctx());
            Self::render_eraser_tool(state, ui);
        });

        Self::render_windows(state, ui);
    }

    // 各種機能ウィンドウの描画
    fn render_windows(state: &mut AppState, ui: &mut Ui) {
        TurnFeature::render(state, ui);
        CommsFeature::render(state, ui);
        LightsFeature::render(state, ui);
    }

    fn render_eraser_tool(state: &mut AppState, ui: &mut Ui) {
        // 消しゴムツールの描画位置を計算
        let main_rect = ui.max_rect(); // メイン領域全体のサイズ
        let tool_size = Vec2::new(60.0, 60.0);
        let margin_x = 10.0;
        let margin_y = 100.0;

        // 左下から10px離れた位置に配置
        let tool_pos = Pos2::new(
            main_rect.min.x + margin_x,
            main_rect.max.y - tool_size.y - margin_y,
        );

        let tool_rect = Rect::from_min_size(tool_pos, tool_size);

        ui.scope_builder(UiBuilder::new().max_rect(tool_rect), |ui| {
            EraserFeature::render(state, ui);
        });
    }
}
