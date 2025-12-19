mod constants;
mod view;

use crate::state::{Actions, AppState, WindowAction};
use egui::Ui;
use view::CommsView;

pub struct CommsFeature;

impl CommsFeature {
    pub fn render(state: &AppState, ui: &mut Ui) {
        let res = CommsView::render(state, ui);

        if let Some(player) = res.click_player {
            let actions = Actions::new(state);
            actions.handle_window(WindowAction::ToggleComms(player.id));
        }
    }
}
