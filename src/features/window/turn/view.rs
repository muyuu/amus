use egui::*;

use crate::{state::AppState};
use super::TurnConstants;

pub struct TurnViewResult {
    pub selected_wave_index: Option<usize>,
}

pub struct TurnView;

impl TurnView {
    pub fn render(state: &AppState, ui: &mut egui::Ui, total_waves: usize) -> TurnViewResult {
        let mut result = TurnViewResult {
            selected_wave_index: None,
        };

        let default_pos = Pos2 {
            x: TurnConstants::TURN_PANEL_DEFAULT_POS_OFFSET_X,
            y: TurnConstants::TURN_PANEL_DEFAULT_POS_OFFSET_Y,
        };
        Window::new("ターン")
            .default_pos(default_pos)
            .show(ui.ctx(), |ui| {
                Grid::new("turn_grid")
                    // 現時点ではこの指定は列を強制するわけではないが内部的にあった方がいいかもだから入れてる
                    .num_columns(TurnConstants::TURN_GRID_ROWS)
                    .spacing([
                        TurnConstants::TURN_GRID_SPACE,
                        TurnConstants::TURN_GRID_SPACE,
                    ])
                    .show(ui, |ui| {
                        for turn_index in 0..total_waves {
                            let turn_number = turn_index + 1;
                            let is_current = state.current_wave_index() == turn_index;

                            let button =
                                TurnView::create_button(ui, &turn_number.to_string(), is_current);

                            // 4列で改行
                            if (turn_index + 1) % TurnConstants::TURN_GRID_ROWS == 0 {
                                ui.end_row();
                            }

                            let clicked = button.clicked();
                            if clicked {
                                let selected_wave_index = Some(turn_index);
                                result.selected_wave_index = selected_wave_index;
                            }

                            // ホバー時にカーソルをポインターに変更
                            if button.hovered() {
                                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                            }
                        }
                    });
            });

        result
    }

    fn create_button(ui: &mut Ui, text: &str, is_current: bool) -> Response {
        let bg = if is_current {
            TurnConstants::TURN_BUTTON_BG_COLOR_HIGHLIGHT
        } else {
            TurnConstants::TURN_BUTTON_BG_COLOR
        };

        let txt = RichText::new(text)
            .color(Color32::WHITE)
            .size(TurnConstants::TURN_BUTTON_FONT_SIZE);
        let button = Button::new(txt)
            .min_size(Vec2 { x: 30.0, y: 20.0 })
            .fill(bg);
        ui.add_enabled(!is_current, button)
    }
}
