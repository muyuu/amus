use super::common_texts::CommonTexts;
use crate::game::state::AppState;
use crate::i18n::keys::*;
use egui::*;

pub struct Sidebar;

impl Sidebar {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        let texts = SidebarTexts::get(state);
        let common = CommonTexts::get(state);

        ui.heading(&texts.turn_management);

        let mut selected_wave = None;
        let mut add_new_wave = false;
        let mut selected_user = None;

        if let Some(game) = &state.game {
            // ターン一覧
            ScrollArea::vertical()
                .id_source("turns_scroll")
                .show(ui, |ui| {
                    for (i, _wave) in game.waves.iter().enumerate() {
                        let label = if i == state.current_wave_index {
                            format!("{} {}", texts.current_turn_prefix, i + 1)
                        } else {
                            format!("{} {}", common.label_turn_prefix, i + 1)
                        };

                        if ui
                            .selectable_label(i == state.current_wave_index, label)
                            .clicked()
                        {
                            selected_wave = Some(i);
                        }
                    }
                });

            ui.separator();

            if ui.button(&texts.add_new_turn).clicked() {
                add_new_wave = true;
            }

            ui.separator();
            ui.heading(&texts.players);

            // プレイヤー一覧
            ScrollArea::vertical()
                .id_source("players_scroll")
                .show(ui, |ui| {
                    for (i, user) in game.users.iter().enumerate() {
                        let color = user.color.to_egui_color();
                        let mut label = RichText::new(&user.name).color(color);

                        if !user.alive {
                            label = label.strikethrough();
                        }

                        let is_selected = state.selected_user_id == Some(i);
                        if ui.selectable_label(is_selected, label).clicked() {
                            selected_user = Some(i);
                        }
                    }
                });
        } else {
            ui.label(&texts.start_game_prompt);
            if ui.button(&common.button_new_game).clicked() {
                state.start_new_game();
            }
        }

        // 状態の更新を最後に行う
        if let Some(wave_idx) = selected_wave {
            state.select_wave(wave_idx);
        }
        if add_new_wave {
            state.add_new_wave();
        }
        if let Some(user_idx) = selected_user {
            state.select_user(user_idx);
        }
    }
}

// サイドバー固有のテキスト
struct SidebarTexts {
    pub turn_management: String,
    pub add_new_turn: String,
    pub players: String,
    pub start_game_prompt: String,
    pub current_turn_prefix: String,
}

impl SidebarTexts {
    fn get(state: &AppState) -> Self {
        Self {
            turn_management: state.t(SIDEBAR_TURN_MANAGEMENT).to_string(),
            add_new_turn: state.t(SIDEBAR_ADD_NEW_TURN).to_string(),
            players: state.t(SIDEBAR_PLAYERS).to_string(),
            start_game_prompt: state.t(SIDEBAR_START_GAME_PROMPT).to_string(),
            current_turn_prefix: state.t(SIDEBAR_CURRENT_TURN_PREFIX).to_string(),
        }
    }
}
