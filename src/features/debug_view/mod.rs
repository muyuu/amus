mod view;

use crate::state::AppState;
use egui::Context;
use view::DebugView;

pub struct DebugFeature;

impl DebugFeature {
    pub fn render(state: &AppState, ctx: &Context) {
        let slices = state.slices();
        let render_flag = slices.ui().show_debug_view();
        DebugView::render(&slices, render_flag, ctx);
    }
}
