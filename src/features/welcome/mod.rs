mod view;

use crate::state::{Actions, AppState};
use egui::Ui;
use view::WelcomeView;

pub struct WelcomeFeature;

impl WelcomeFeature {
    pub fn render(ui: &mut Ui, state: &mut AppState) {
        let game_actions = WelcomeView::render(ui, state);
        let mut actions = Actions::new(state);
        for action in game_actions {
            actions.handle_game(action);
        }
    }
}
