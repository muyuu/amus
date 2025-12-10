use egui::*;

use crate::{
    components::{grid, tile, window},
    constants::GridIds,
    i18n::keys::*,
    models::Player,
    state::AppState,
};

use super::O2Constants;

pub struct O2ViewResult {
    pub click_player: Option<Player>,
}

pub struct O2View;

impl O2View {
    pub fn render(state: &AppState, ui: &mut Ui) -> O2ViewResult {
        let text = O2Text::get(state);
        let mut result = O2ViewResult { click_player: None };

        let pos = Pos2 {
            x: O2Constants::PANEL_DEFAULT_POS_OFFSET_X,
            y: O2Constants::PANEL_DEFAULT_POS_OFFSET_Y,
        };
        let ctx = ui.ctx();

        window(ctx, &text.title, None, Some(pos), |ui: &mut Ui| {
            grid(ui, GridIds::O2, |ui| {
                let players = match state.players() {
                    Some(p) => p,
                    None => return,
                };

                let r = Self::render_players(ui, &players);
                if let Some(player) = r.click_player {
                    result.click_player = Some(player);
                }
            });
        });

        result
    }

    pub fn render_players(ui: &mut egui::Ui, players: &[Player]) -> O2ViewResult {
        let mut result = O2ViewResult { click_player: None };

        for (index, player) in players.iter().enumerate() {
            let res = tile::with_mark(
                ui,
                &tile::WithMarkConf {
                    color: player.color.to_egui_color(),
                    label: Some(player.name.clone()),
                    size: None,
                    is_selected: false,
                    is_dragging: false,
                    marked: player.resolved_o2,
                },
            );

            if res.clicked {
                result.click_player = Some(player.clone());
            }

            // 4列で改行
            if (index + 1) % O2Constants::GRID_ROWS == 0 {
                ui.end_row();
            }
        }

        result
    }
}

struct O2Text {
    title: String,
}
impl O2Text {
    fn get(state: &AppState) -> Self {
        Self {
            title: state.t(SABOTAGE_O2).to_string(),
        }
    }
}
