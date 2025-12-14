use egui::*;

use crate::{i18n::keys::ERASER_BUTTON, state::AppState};

pub struct EraserView;

impl EraserView {
    pub fn render(state: &AppState, ui: &mut Ui) -> bool {
        let mut result = false;

        // 消しゴムツールの描画位置を計算
        let main_rect = ui.available_rect_before_wrap();
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
            result = Self::render_button(state, ui);
        });

        result
    }
    fn render_button(state: &AppState, ui: &mut Ui) -> bool {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(40.0, 40.0), Sense::click_and_drag());

        ui.painter().rect_filled(rect, 4.0, Color32::PLACEHOLDER);

        if state.erase_mode() {
            ui.painter()
                .rect_stroke(rect, 4.0, (2.0, Color32::WHITE), StrokeKind::Inside);
        }
        ui.label(RichText::new(state.t(ERASER_BUTTON)).color(Color32::WHITE));
        response.clicked()
    }
}
