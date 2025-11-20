use super::common_texts::CommonTexts;
use crate::i18n::keys::*;
use crate::models::Color;
use crate::state::AppState;

pub struct SetupDialog;

impl SetupDialog {
    pub fn show(state: &mut AppState, ctx: &egui::Context) {
        let texts = SetupTexts::get(state);
        let common = CommonTexts::get(state);

        egui::Window::new(&texts.title)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 700.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(&texts.area_selection);
                        ui.add_space(8.0); // ヘッディング後の余白

                        // エリア一覧（ComboBox形式）
                        let areas = vec![
                            ("The Skeld", "skeld"),
                            ("Mira HQ", "mira"),
                            ("Polus", "polus"),
                            ("Airship", "airship"),
                        ];

                        egui::ComboBox::from_label("")
                            .selected_text(&state.setup_state.selected_area_name)
                            .show_ui(ui, |ui| {
                                for (name, id) in &areas {
                                    if ui
                                        .selectable_label(
                                            state.setup_state.selected_area_id == *id,
                                            *name,
                                        )
                                        .clicked()
                                    {
                                        state.setup_state.selected_area_id = id.to_string();
                                        state.setup_state.selected_area_name = name.to_string();
                                    }
                                }
                            });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.heading(&texts.player_settings);
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(&texts.player_count);
                            let mut new_count = state.setup_state.player_count;
                            if ui.add(egui::Slider::new(&mut new_count, 4..=18)).changed() {
                                state.adjust_player_count(new_count);
                            }
                        });

                        ui.add_space(8.0);

                        // プレイヤー一覧の編集
                        let available_colors = Color::all();
                        let player_count = state.setup_state.player_count;

                        for i in 0..player_count {
                            if i < state.setup_state.players.len() {
                                ui.horizontal(|ui| {
                                    ui.label(&texts.player_name);
                                    ui.add_space(4.0);
                                    ui.text_edit_singleline(&mut state.setup_state.players[i].name);

                                    ui.add_space(8.0);
                                    ui.label(&texts.player_color);
                                    ui.add_space(4.0);

                                    // 現在の色のプレビューをComboBoxの前に表示（アライメント調整）
                                    let selected_color = &mut state.setup_state.players[i].color;

                                    // 色プレビューのサイズと位置を調整
                                    let (rect, _response) = ui.allocate_exact_size(
                                        egui::Vec2::new(20.0, 20.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        2.0,
                                        selected_color.to_egui_color(),
                                    );

                                    ui.add_space(4.0);

                                    // 色選択のためのComboBox
                                    egui::ComboBox::from_id_source(format!("color_combo_{}", i))
                                        .selected_text(selected_color.name())
                                        .show_ui(ui, |ui| {
                                            for color in &available_colors {
                                                ui.horizontal(|ui| {
                                                    // 色のプレビューを表示
                                                    let color_rect = egui::Rect::from_min_size(
                                                        ui.next_widget_position(),
                                                        egui::Vec2::new(12.0, 12.0),
                                                    );
                                                    ui.allocate_rect(
                                                        color_rect,
                                                        egui::Sense::hover(),
                                                    );
                                                    ui.painter().rect_filled(
                                                        color_rect,
                                                        2.0,
                                                        color.to_egui_color(),
                                                    );

                                                    if ui
                                                        .selectable_label(
                                                            *selected_color == *color,
                                                            color.name(),
                                                        )
                                                        .clicked()
                                                    {
                                                        *selected_color = color.clone();
                                                    }
                                                });
                                            }
                                        });
                                });
                                ui.add_space(4.0); // 各プレイヤー行間の余白
                            }
                        }

                        ui.add_space(16.0); // プレイヤーリストとボタン間の余白
                        ui.separator();
                        ui.add_space(12.0); // セパレーター後の余白

                        ui.horizontal(|ui| {
                            if ui.button(&common.button_start_game).clicked() {
                                state.create_game_from_setup();
                            }

                            if ui.button(&common.button_cancel).clicked() {
                                state.cancel_setup();
                            }
                        });
                    });
                });
            });
    }
}

// セットアップダイアログ固有のテキスト
struct SetupTexts {
    pub title: String,
    pub area_selection: String,
    pub player_settings: String,
    pub player_count: String,
    pub player_name: String,
    pub player_color: String,
}

impl SetupTexts {
    fn get(state: &AppState) -> Self {
        Self {
            title: state.t(SETUP_TITLE).to_string(),
            area_selection: state.t(SETUP_AREA_SELECTION).to_string(),
            player_settings: state.t(SETUP_PLAYER_SETTINGS).to_string(),
            player_count: state.t(SETUP_PLAYER_COUNT).to_string(),
            player_name: state.t(SETUP_PLAYER_NAME).to_string(),
            player_color: state.t(SETUP_PLAYER_COLOR).to_string(),
        }
    }
}
