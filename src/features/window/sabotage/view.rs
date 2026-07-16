use egui::*;

use crate::{
    components::{grid, tile, window},
    models::{Player, Sabotage},
    state::Slices,
};

use crate::common::ui_color_adapter::to_egui_color;

/// 1 行あたりのタイル数（この数で改行する）。
const GRID_COLUMNS: usize = 4;

/// パネル初期表示位置の Y オフセット（全種別共通）。
const PANEL_DEFAULT_POS_OFFSET_Y: f32 = 10.0;

pub struct SabotageViewResult {
    pub click_player: Option<Player>,
}

pub struct SabotageWindowView;

impl SabotageWindowView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui, kind: Sabotage) -> SabotageViewResult {
        let title = slices.t(kind.title_key());
        let mut result = SabotageViewResult { click_player: None };

        let pos = Pos2 {
            x: kind.panel_offset_x(),
            y: PANEL_DEFAULT_POS_OFFSET_Y,
        };

        let ctx = ui.ctx();

        window(ctx, &title, None, Some(pos), |ui: &mut Ui| {
            grid(ui, kind.grid_id(), |ui| {
                let player_slice = slices.player();
                let players = match player_slice.players() {
                    Some(p) => p,
                    None => return,
                };

                let r = Self::render_players(ui, players, kind);
                if let Some(player) = r.click_player {
                    result.click_player = Some(player);
                }
            });
        });

        result
    }

    fn render_players(ui: &mut Ui, players: &[Player], kind: Sabotage) -> SabotageViewResult {
        let mut result = SabotageViewResult { click_player: None };

        for (index, player) in players.iter().enumerate() {
            let res = tile::with_mark(
                ui,
                &tile::WithMarkConf {
                    color: to_egui_color(&player.color),
                    label: Some(player.name.clone()),
                    size: None,
                    is_selected: false,
                    is_dragging: false,
                    marked: player.is_resolved(kind),
                },
            );

            if res.clicked {
                result.click_player = Some(player.clone());
            }

            if (index + 1) % GRID_COLUMNS == 0 {
                ui.end_row();
            }
        }

        result
    }
}
