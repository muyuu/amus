pub mod constants;
pub mod interaction;
pub mod view;

pub use constants::LightsConstants;
use egui::Ui;
pub use interaction::LightsInteraction;
pub use view::LightsView;

use crate::state::AppState;

pub struct LightsFeature;

impl LightsFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = LightsView::render(state, ui);
        LightsInteraction::handle(state, &res);
    }
}
