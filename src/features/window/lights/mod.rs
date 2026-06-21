mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{AppState, WindowAction};
use egui::Ui;
use view::LightsView;

pub struct LightsFeature;

impl LightsFeature {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<AppAction> {
        let slices = state.slices();
        let res = LightsView::render(&slices, ui);

        res.click_player
            .map(|player| AppAction::Window(WindowAction::ToggleLights(player.id)))
            .into_iter()
            .collect()
    }
}
