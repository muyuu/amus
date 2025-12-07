pub mod constants;
pub mod interaction;
pub mod view;

pub use constants::ReactorConstants;
use egui::Ui;
pub use interaction::ReactorInteraction;
pub use view::ReactorView;

use crate::state::AppState;

pub struct ReactorFeature;

impl ReactorFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = ReactorView::render(state, ui);
        ReactorInteraction::handle(state, &res);
    }
}
