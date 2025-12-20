mod view;

use crate::state::AppState;
use egui::{Response, Ui};
use view::MapView;

pub struct MapFeature;

impl MapFeature {
    pub fn render(state: &AppState, response: &Response, ui: &mut Ui) {
        let slices = state.slices();
        MapView::render(&slices, response, ui);
    }
}
