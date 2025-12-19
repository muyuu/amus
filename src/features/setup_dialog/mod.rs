mod view;

use egui::*;
use view::SetupView;

use crate::state::{Actions, AppState};

pub struct SetupDialogFeature;

impl SetupDialogFeature {
    pub fn render(state: &AppState, ctx: &Context) {
        let setup_actions = SetupView::render(state, ctx);
        let actions = Actions::new(state);
        for action in setup_actions {
            actions.handle_setup(action);
        }
    }
}
