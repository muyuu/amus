mod constants;
mod view;

use crate::state::{Actions, AppState, WindowAction};
use egui::Ui;
use view::LightsView;

pub struct LightsFeature;

impl LightsFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = LightsView::render(state, ui);

        if let Some(player) = res.click_player {
            let mut actions = Actions::new(state);
            actions.handle_window(WindowAction::ToggleLights(player.id));
        }
    }
}
