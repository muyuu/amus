pub mod constants;
pub mod interaction;
pub mod view;

pub use constants::LocationConstants;
pub use interaction::LocationInteraction;
pub use view::LocationView;

use crate::state::AppState;
use egui::*;

pub struct LocationFeature;

impl LocationFeature {
    pub fn render(state: &mut AppState, response: &Response, ui: &mut Ui) {
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        LocationInteraction::handle_interactions(state, response, ui.ctx());
        LocationView::render(state, &painter, rect);
        // 位置関連の初期化や設定があればここに追加
    }
}
