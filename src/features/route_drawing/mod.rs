use crate::state::AppState;

mod interaction;
mod view;

pub struct RouteDrawingFeature;

impl RouteDrawingFeature {
    pub fn render(state: &mut AppState, response: &egui::Response, ui: &mut egui::Ui) {
        view::RouteDrawingView::render(state, response, ui);
        interaction::RouteDrawingInteraction::handle(state, response);
    }
}
