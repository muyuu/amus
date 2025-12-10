mod constants;
pub mod interaction;
pub mod view;

use crate::state::AppState;
use egui::*;
pub use interaction::PlayerInfoInteraction;
pub use view::PlayerInfoView;

pub struct PlayerInfoFeature;

impl PlayerInfoFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = PlayerInfoView::render(state, ui);
        PlayerInfoInteraction::handle(state, &res);
    }
}
