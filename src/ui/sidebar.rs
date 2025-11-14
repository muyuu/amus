use crate::game::state::AppState;
use egui::*;

pub struct Sidebar;

impl Sidebar {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        ui.heading("ターン管理");

        let mut selected_wave = None;
        let mut add_new_wave = false;
        let mut selected_user = None;

        if let Some(game) = &state.game {
            // ターン一覧
            ScrollArea::vertical().show(ui, |ui| {
                for (i, _wave) in game.waves.iter().enumerate() {
                    let label = if i == state.current_wave_index {
                        format!("▶ ターン {}", i + 1)
                    } else {
                        format!("ターン {}", i + 1)
                    };

                    if ui.selectable_label(i == state.current_wave_index, label).clicked() {
                        selected_wave = Some(i);
                    }
                }
            });

            ui.separator();

            if ui.button("新しいターンを追加").clicked() {
                add_new_wave = true;
            }

            ui.separator();
            ui.heading("プレイヤー");

            // プレイヤー一覧
            ScrollArea::vertical().show(ui, |ui| {
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
            ui.label("ゲームを開始してください");
            if ui.button("新規ゲーム").clicked() {
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