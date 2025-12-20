mod constants;
mod view;

use crate::state::{Actions, AppState, WindowAction};
use egui::Ui;
use view::ReactorView;

pub struct ReactorFeature;

impl ReactorFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let slices = state.slices();
        let res = ReactorView::render(&slices, ui);

        if let Some(player) = res.click_player {
            let mut actions = Actions::new(state);
            actions.handle_window(WindowAction::ToggleReactor(player.id));
        }
    }
}
