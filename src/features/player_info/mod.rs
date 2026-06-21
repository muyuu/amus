mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::Slices;
use egui::*;
use view::PlayerInfoView;

pub struct PlayerInfoFeature;

impl PlayerInfoFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        PlayerInfoView::render(slices, ui)
            .into_iter()
            .map(AppAction::PlayerInfo)
            .collect()
    }
}
