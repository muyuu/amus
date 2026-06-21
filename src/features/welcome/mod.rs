mod view;

use crate::app_action::AppAction;
use crate::state::AppState;
use egui::Ui;
use view::WelcomeView;

pub struct WelcomeFeature;

impl WelcomeFeature {
    pub fn render(ui: &mut Ui, state: &AppState) -> Vec<AppAction> {
        let slices = state.slices();
        WelcomeView::render(ui, &slices)
            .into_iter()
            .map(AppAction::Game)
            .collect()
    }
}
