mod view;

use egui::*;
use view::SetupView;

use crate::state::{Actions, AppState};

pub struct SetupDialogFeature;

impl SetupDialogFeature {
    pub fn render(state: &mut AppState, ctx: &Context) {
        let slices = state.slices();
        let setup_actions = SetupView::render(&slices, ctx);
        let mut actions = Actions::new(state);
        for action in setup_actions {
            actions.handle_setup(action);
        }
    }
}
