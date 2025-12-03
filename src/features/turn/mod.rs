pub mod constants;
pub mod interaction;
pub mod view;

use egui::Ui;
pub use interaction::TurnInteraction;
pub use view::TurnView;

use crate::state::AppState;

pub struct TurnFeature;

impl TurnFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let game = match state.game() {
            Some(game) => game,
            None => return,
        };

        let res = TurnView::render(state, ui, game.waves.len());
        TurnInteraction::handle(state, &res);
    }
}
