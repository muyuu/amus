use egui::*;

use crate::{
    components::{grid, tile, window},
    constants::GridIds,
    i18n::keys::*,
    models::Player,
    state::Slices,
};

use super::constants::CommsConstants;
use crate::common::ui_color_adapter::to_egui_color;

pub struct CommsViewResult {
    pub click_player: Option<Player>,
}

pub struct CommsView;

impl CommsView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> CommsViewResult {
        let text = CommsText::get(slices);
        let mut result = CommsViewResult { click_player: None };

        let pos = Pos2 {
            x: CommsConstants::PANEL_DEFAULT_POS_OFFSET_X,
            y: CommsConstants::PANEL_DEFAULT_POS_OFFSET_Y,
        };

        let ctx = ui.ctx();

        window(ctx, &text.title, None, Some(pos), |ui: &mut Ui| {
            grid(ui, GridIds::COMMS, |ui| {
                let player_slice = slices.player();
                let players = match player_slice.players() {
                    Some(p) => p,
                    None => return,
                };

                let r = Self::render_players(ui, players);
                if let Some(player) = r.click_player {
                    result.click_player = Some(player);
                }
            });
        });

        result
    }

    pub fn render_players(ui: &mut egui::Ui, players: &[Player]) -> CommsViewResult {
        let mut result = CommsViewResult { click_player: None };

        for (index, player) in players.iter().enumerate() {
            let res = tile::with_mark(
                ui,
                &tile::WithMarkConf {
                    color: to_egui_color(&player.color),
                    label: Some(player.name.clone()),
                    size: None,
                    is_selected: false,
                    is_dragging: false,
                    marked: player.resolved_comms,
                },
            );

            if res.clicked {
                result.click_player = Some(player.clone());
            }

            // 4列で改行
            if (index + 1) % CommsConstants::GRID_ROWS == 0 {
                ui.end_row();
            }
        }

        result
    }
}

struct CommsText {
    title: String,
}
impl CommsText {
    fn get(slices: &Slices<'_>) -> Self {
        Self {
            title: slices.t(SABOTAGE_COMMS),
        }
    }
}
