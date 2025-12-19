mod view;

use crate::state::AppState;
use egui::Context;
use view::DebugView;

pub struct DebugFeature;

impl DebugFeature {
    pub fn render(state: &AppState, render_flag: bool, ctx: &Context) {
        DebugView::render(state, render_flag, ctx);
    }
}
