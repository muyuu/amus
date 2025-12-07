pub mod constants;
pub mod interaction;
pub mod view;

use egui::Ui;

pub use constants::CommsConstants;
pub use interaction::CommsInteraction;
pub use view::CommsView;

use crate::state::AppState;

pub struct CommsFeature;

impl CommsFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = CommsView::render(state, ui);
        CommsInteraction::handle(state, &res);
    }
}
