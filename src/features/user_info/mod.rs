pub mod view;

use egui::*;
pub use view::UserInfoView;

use crate::state::AppState;

pub struct UserInfoFeature;

impl UserInfoFeature {
    pub fn render(state: &AppState, ui: &mut Ui) {
        UserInfoView::render(state, ui);
    }
}
