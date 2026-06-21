mod constants;
mod view;

use crate::app_action::AppAction;
use crate::state::{Slices, WindowAction};
use egui::Ui;
use view::CommsView;

pub struct CommsFeature;

impl CommsFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        let res = CommsView::render(slices, ui);

        res.click_player
            .map(|player| AppAction::Window(WindowAction::ToggleComms(player.id)))
            .into_iter()
            .collect()
    }
}
