mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{Slices, WindowAction};
use egui::Ui;
use view::ReactorView;

pub struct ReactorFeature;

impl ReactorFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        let res = ReactorView::render(slices, ui);

        res.click_player
            .map(|player| AppAction::Window(WindowAction::ToggleReactor(player.id)))
            .into_iter()
            .collect()
    }
}
