pub mod interaction;
pub mod view;

use crate::state::AppState;
use egui::*;
pub use interaction::UserInfoInteraction;
pub use view::UserInfoView;

pub struct UserInfoFeature;

impl UserInfoFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let res = UserInfoView::render(state, ui);
        UserInfoInteraction::handle(state, &res);
    }
}
