mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{AppState, WindowAction};
use egui::Ui;
use view::ReactorView;

pub struct ReactorFeature;

impl ReactorFeature {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<AppAction> {
        let slices = state.slices();
        let res = ReactorView::render(&slices, ui);

        res.click_player
            .map(|player| AppAction::Window(WindowAction::ToggleReactor(player.id)))
            .into_iter()
            .collect()
    }
}
