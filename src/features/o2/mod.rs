pub mod constants;
pub mod interaction;
pub mod view;

pub use constants::O2Constants;
use egui::Ui;
pub use interaction::O2Interaction;
pub use view::O2View;

use crate::state::AppState;

pub struct O2Feature;

impl O2Feature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = O2View::render(state, ui);
        O2Interaction::handle(state, &res);
    }
}
