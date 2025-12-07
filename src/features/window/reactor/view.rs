use egui::*;

use crate::{
    components::player_rect, constants::GridIds, i18n::keys::*, models::Player, state::AppState,
};

use super::ReactorConstants;

pub struct ReactorViewResult {
    pub click_player: Option<Player>,
}

pub struct ReactorView;

impl ReactorView {
    pub fn render(state: &AppState, ui: &mut egui::Ui) -> ReactorViewResult {
        let text = ReactorText::get(state);
        let mut result = ReactorViewResult { click_player: None };

        let default_pos = Pos2 {
            x: ReactorConstants::PANEL_DEFAULT_POS_OFFSET_X,
            y: ReactorConstants::PANEL_DEFAULT_POS_OFFSET_Y,
        };
        Window::new(text.title)
            .default_pos(default_pos)
            .show(ui.ctx(), |ui| {
                Grid::new(GridIds::REACTOR)
                    // 現時点ではこの指定は列を強制するわけではないが内部的にあった方がいいかもだから入れてる
                    .num_columns(ReactorConstants::GRID_ROWS)
                    .spacing([ReactorConstants::GRID_SPACE, ReactorConstants::GRID_SPACE])
                    .show(ui, |ui| {
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

    pub fn render_players(ui: &mut egui::Ui, players: &[Player]) -> ReactorViewResult {
        let mut result = ReactorViewResult { click_player: None };

        for (index, player) in players.iter().enumerate() {
            let res = player_rect(ui, player, false, false);

            // resolved_reactorがtrueの場合、中抜きの丸を表示
            if player.resolved_reactor {
                let center = res.rect.center();
                let radius = res.rect.width().min(res.rect.height()) * 0.3;

                // プレイヤーの色が赤の場合は白、それ以外は赤
                let circle_color = if player.color == crate::models::color::Color::Red {
                    Color32::WHITE
                } else {
                    Color32::RED
                };

                ui.painter().circle_stroke(
                    center,
                    radius,
                    Stroke::new(ReactorConstants::RESOLVED_MARK_SIZE, circle_color),
                );
            }

            if res.clicked {
                result.click_player = Some(player.clone());
            }

            // 4列で改行
            if (index + 1) % ReactorConstants::GRID_ROWS == 0 {
                ui.end_row();
            }
        }

        result
    }
}

struct ReactorText {
    title: String,
}
impl ReactorText {
    fn get(state: &AppState) -> Self {
        Self {
            title: state.t(SABOTAGE_REACTOR).to_string(),
        }
    }
}
