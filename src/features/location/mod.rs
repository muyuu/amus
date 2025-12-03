pub mod constants;
pub mod interaction;
pub mod view;

pub use interaction::LocationInteraction;
pub use view::LocationView;

use crate::state::AppState;
use egui::*;

pub struct LocationFeature;

impl LocationFeature {
    pub fn render(state: &mut AppState, response: &Response, ui: &mut Ui) {
        let result = LocationView::render(state, ui);
        LocationInteraction::handle(state, &result, response, ui);
    }
}
