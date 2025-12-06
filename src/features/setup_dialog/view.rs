use egui::*;

use crate::common::{texts, CommonTexts};
use crate::i18n::keys::*;
use crate::models::player::PlayerId;
use crate::models::{Area, Color};
use crate::state::AppState;

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
                    // エリア選択
                    let area_res = Self::render_area(state, ui, &texts);
                    if let Some(area) = area_res {
                        result.select_area = Some(area);
                    }

                    ui.add_space(12.0);

                    // ユーザー人数調整
                    let user_count_res = Self::render_user_count(state, ui, &texts);
                    if let Some(count) = user_count_res {
                        result.player_count = Some(count);
                    }

                    ui.add_space(12.0);

                    // プレイヤー情報編集
                    let player_info_res = Self::render_player_info(state, ui, &texts);
                    if let Some((id, name, color)) = player_info_res {
                        result.update_player = Some((id, name, color));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // ボタン群
                    let (start_game, cancel) = Self::render_buttons(ui, &common);
                    result.start_game = start_game;
                    result.cancel = cancel;
                });
            });

        result
    }

    fn render_area(state: &AppState, ui: &mut Ui, texts: &SetupTexts) -> Option<Area> {
        let mut result = None;

        ui.horizontal(|ui| {
            ui.label(&texts.area_selection);
            ui.add_space(8.0);

            let setup_state = state.setup_state().clone();
            ComboBox::from_label("")
                .selected_text(setup_state.selected_area.name())
                .show_ui(ui, |ui| {
                    for area in Area::all() {
                        if ui
                            .selectable_label(
                                state.setup_state().selected_area == area,
                                &*area.name(),
                            )
                            .clicked()
                        {
                            result = Some(area.clone());
                        }
                    }
                });
        });

        result
    }

    fn render_user_count(state: &AppState, ui: &mut Ui, texts: &SetupTexts) -> Option<usize> {
        let mut result: Option<usize> = None;
        ui.horizontal(|ui| {
            ui.label(&texts.player_count);
            let mut new_count = state.setup_state().player_count;
            if ui.add(Slider::new(&mut new_count, 4..=18)).changed() {
                result = Some(new_count);
            }
        });

        result
    }

    fn render_player_info(
        state: &AppState,
        ui: &mut Ui,
        texts: &SetupTexts,
    ) -> Option<(PlayerId, String, Color)> {
        let mut result = None;

        let available_colors = Color::all();
        let player_count = state.setup_state().player_count;

        for i in 0..player_count {
            ui.horizontal(|ui| {
                ui.label(&texts.player_name);
                ui.add_space(4.0);

                let player = &state.setup_state().players[i].clone();
                let mut name = player.name.clone();
                let res = ui.text_edit_singleline(&mut name);

                ui.add_space(8.0);

                // 選択中の色と同じ場合は別の色を表示する
                let (selected_id, selected_color) = match state.selected_player_color() {
                    Some((id, color)) => (Some(id), Some(color)),
                    None => (None, None),
                };
                let player_color = if selected_id != Some(player.id)
                    && selected_color == Some(player.color.clone())
                {
                    let a = available_colors
                        .iter()
                        .find(|c| **c != player.color)
                        .cloned()
                        .unwrap_or(player.color.clone());

                    result = Some((player.id, name.clone(), a.clone()));

                    a
                } else {
                    player.color.clone()
                };

                // 色プレビューのサイズと位置を調整
                let (rect, _response) =
                    ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, 2.0, player_color.to_egui_color());

                if res.changed() {
                    result = Some((player.id, name, player_color.clone()));
                }

                ui.add_space(4.0);

                // 色選択のためのComboBox
                ComboBox::from_id_salt(format!("color_combo_{}", i))
                    .selected_text(player_color.name())
                    .show_ui(ui, |ui| {
                        for color in &available_colors {
                            let (rect, response) = ui.allocate_exact_size(
                                ui.available_size_before_wrap(),
                                Sense::click(),
                            );

                            if response.clicked() {
                                result = Some((player.id, player.name.clone(), color.clone()));
                            }

                            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                                ui.horizontal(|ui| {
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

                                    ui.selectable_label(player_color == *color, color.name())
                                });
                            });
                        }
                    });
            });
            ui.add_space(4.0);
        }
        result
    }

    fn render_buttons(ui: &mut Ui, texts: &CommonTexts) -> (bool, bool) {
        let mut result = (false, false);
        ui.horizontal(|ui| {
            if ui.button(&texts.button_start_game).clicked() {
                result = (true, false);
            }

            if ui.button(&texts.button_cancel).clicked() {
                result = (false, true);
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
