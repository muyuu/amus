mod constants;
mod view;

use constants::PlayerInfoConstants;
use view::PlayerListView;

use crate::app_action::AppAction;
use crate::state::Slices;
use egui::Ui;

/// プレイヤーリスト機能
pub struct PlayerListFeature;

impl PlayerListFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        PlayerListView::render(slices, ui)
            .into_iter()
            .map(AppAction::Player)
            .collect()
    }
}
