mod view;

use crate::app_action::AppAction;
use crate::state::Slices;
use egui::Ui;
use view::DebugView;

pub struct DebugFeature;

impl DebugFeature {
    // 描画のみ（操作なし）。ctx は ui.ctx() から取る
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        let render_flag = slices.ui().show_debug_view();
        DebugView::render(slices, render_flag, ui.ctx());
        Vec::new()
    }
}
