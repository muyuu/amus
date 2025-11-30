use egui::*;

use crate::state::AppState;

mod interaction;
mod view;

pub struct RouteDrawingFeature;

impl RouteDrawingFeature {
    pub fn render(state: &mut AppState, response: &Response, ui: &mut Ui) {
        view::RouteDrawingView::render(state, response, ui);
        interaction::RouteDrawingInteraction::handle(state, response);
    }
}
