mod view;

use crate::app_action::AppAction;
use crate::models::Sabotage;
use crate::state::{PlayerAction, Slices};
use egui::Ui;
use view::SabotageWindowView;

pub struct SabotageFeature;

impl SabotageFeature {
    pub fn render(slices: &Slices, ui: &mut Ui, kind: Sabotage) -> Vec<AppAction> {
        let res = SabotageWindowView::render(slices, ui, kind);

        res.click_player
            .map(|player| AppAction::Player(PlayerAction::ToggleSabotage(kind, player.id)))
            .into_iter()
            .collect()
    }
}
