mod view;

use egui::*;
use view::SetupView;

use crate::app_action::AppAction;
use crate::state::AppState;

pub struct SetupDialogFeature;

impl SetupDialogFeature {
    pub fn render(state: &AppState, ctx: &Context) -> Vec<AppAction> {
        let slices = state.slices();
        SetupView::render(&slices, ctx)
            .into_iter()
            .map(AppAction::Setup)
            .collect()
    }
}
