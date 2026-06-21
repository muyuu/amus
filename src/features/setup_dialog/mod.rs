mod view;

use egui::*;
use view::SetupView;

use crate::app_action::AppAction;
use crate::state::Slices;

pub struct SetupDialogFeature;

impl SetupDialogFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        SetupView::render(slices, ui.ctx())
            .into_iter()
            .map(AppAction::Setup)
            .collect()
    }
}
