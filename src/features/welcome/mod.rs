mod view;

use crate::app_action::AppAction;
use crate::state::Slices;
use egui::Ui;
use view::WelcomeView;

pub struct WelcomeFeature;

impl WelcomeFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        WelcomeView::render(ui, slices)
            .into_iter()
            .map(AppAction::Game)
            .collect()
    }
}
