pub mod interaction;
pub mod view;

use egui::*;
pub use view::SetupView;

use crate::state::AppState;

pub struct SetupDialogFeature;

impl SetupDialogFeature {
    pub fn render(state: &mut AppState, ctx: &Context) {
        let result = SetupView::render(state, ctx);

        if let Some(area) = result.select_area {
            interaction::SetupInteraction::select_area(state, area);
        }

        if let Some(count) = result.player_count {
            interaction::SetupInteraction::adjust_player_count(state, count);
        }

        match result.update_player {
            Some((id, name, color)) => {
                interaction::SetupInteraction::update_player_name(state, id, name);
                interaction::SetupInteraction::update_player_color(state, id, color);
            }
            None => {
                interaction::SetupInteraction::reset_selected_player_color(state);
            }
        }

        if result.start_game {
            interaction::SetupInteraction::start_game(state);
        }

        if result.cancel {
            interaction::SetupInteraction::cancel(state);
        }
    }
}
