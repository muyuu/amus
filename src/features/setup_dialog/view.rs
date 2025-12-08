use egui::*;

use crate::common::CommonTexts;
use crate::i18n::keys::*;
use crate::models::player::PlayerId;
use crate::models::{Area, Color};
use crate::state::{AppState, SetupState};

#[derive(Default)]
pub struct SetupViewResult {
    pub select_area: Option<Area>,
    pub player_count: Option<usize>,
    pub update_player: Option<(PlayerId, String, Color)>,
    pub start_game: bool,
    pub cancel: bool,
}

pub struct SetupView;

impl SetupView {
    pub fn render(state: &AppState, ctx: &Context) -> SetupViewResult {
        let texts = SetupTexts::get(state);
        let common = CommonTexts::get(state);

        let mut result = SetupViewResult::default();

        Window::new(&texts.title)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 700.0])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let area = Self::render_area_combo_box(
                            ui,
                            &state.setup_state(),
                            &texts.area_selection,
                        );
                        if area.is_some() {
                            result.select_area = area;
                        }
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(&texts.player_count);
                        let mut new_count = state.setup_state().player_count;
                        if ui.add(Slider::new(&mut new_count, 4..=18)).changed() {
                            result.player_count = Some(new_count);
                        }
                    });

                    ui.add_space(12.0);

                    // プレイヤー一覧の編集
                    let available_colors = Color::all();
                    let player_count = state.setup_state().player_count;

                    for i in 0..player_count {
                        ui.horizontal(|ui| {
                            ui.label(&texts.player_name);
                            ui.add_space(4.0);

                            let player = &state.setup_state().players[i].clone();
                            let mut name = player.name.clone();
                            if ui.text_edit_singleline(&mut name).changed() {
                                result.update_player =
                                    Some((player.id, name, player.color.clone()));
                            }

                            ui.add_space(8.0);

                            // 現在の色のプレビューをComboBoxの前に表示
                            let selected_color = player.color.clone();

                            // 色プレビューのサイズと位置を調整
                            let (rect, _response) =
                                ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::hover());
                            ui.painter()
                                .rect_filled(rect, 2.0, selected_color.to_egui_color());

                            ui.add_space(4.0);

                            // 色選択のためのComboBox
                            ComboBox::from_id_salt(format!("color_combo_{}", i))
                                .selected_text(selected_color.name())
                                .show_ui(ui, |ui| {
                                    for color in &available_colors {
                                        ui.horizontal(|ui| {
                                            // 色のプレビューを表示
                                            let color_rect = Rect::from_min_size(
                                                ui.next_widget_position(),
                                                Vec2::new(12.0, 12.0),
                                            );
                                            ui.allocate_rect(color_rect, Sense::hover());
                                            ui.painter().rect_filled(
                                                color_rect,
                                                2.0,
                                                color.to_egui_color(),
                                            );

                                            if ui
                                                .selectable_label(
                                                    selected_color == *color,
                                                    color.name(),
                                                )
                                                .clicked()
                                            {
                                                result.update_player = Some((
                                                    player.id,
                                                    player.name.clone(),
                                                    color.clone(),
                                                ));
                                            }
                                        });
                                    }
                                });
                        });
                        ui.add_space(4.0);
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button(&common.button_start_game).clicked() {
                            result.start_game = true;
                        }

                        if ui.button(&common.button_cancel).clicked() {
                            result.cancel = true;
                        }
                    });
                });
            });

        result
    }

    fn render_area_combo_box(ui: &mut Ui, setup_state: &SetupState, text: &str) -> Option<Area> {
        let mut result = None;

        ui.label(text);
        ui.add_space(8.0);

        ComboBox::from_label("")
            .selected_text(setup_state.selected_area.name())
            .show_ui(ui, |ui| {
                for area in Area::all() {
                    if ui
                        .selectable_label(setup_state.selected_area == area, &*area.name())
                        .clicked()
                    {
                        result = Some(area.clone());
                    }
                }
            });
        result
    }
}

// セットアップダイアログ固有のテキスト
struct SetupTexts {
    pub title: String,
    pub area_selection: String,
    pub player_count: String,
    pub player_name: String,
}

impl SetupTexts {
    fn get(state: &AppState) -> Self {
        Self {
            title: state.t(SETUP_TITLE).to_string(),
            area_selection: state.t(SETUP_AREA_SELECTION).to_string(),
            player_count: state.t(SETUP_PLAYER_COUNT).to_string(),
            player_name: state.t(SETUP_PLAYER_NAME).to_string(),
        }
    }
}
