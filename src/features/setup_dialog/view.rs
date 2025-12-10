use egui::*;

use crate::common::{choose_text_color, CommonTexts};
use crate::components::{select_with_contents, select_with_options};
use crate::constants::SelectIds;
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
                    ui.horizontal(|ui| {
                        let area = Self::render_area_combo_box(
                            ui,
                            &mut state.setup_state_mut().selected_area,
                            &texts.area_selection,
                        );
                        if area.is_some() {
                            result.select_area = area;
                        }
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        let r = Self::render_player_amount(ui, state, &texts);
                        if r.player_count.is_some() {
                            result.player_count = r.player_count;
                        }
                    });

                    ui.add_space(12.0);

                    // プレイヤー一覧の編集
                    for i in 0..state.setup_state().player_count {
                        let r = Self::render_player(ui, state, &texts, i, &Color::all());
                        if r.update_player.is_some() {
                            result.update_player = r.update_player;
                        }
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

    fn render_area_combo_box(ui: &mut Ui, selected: &mut Area, text: &str) -> Option<Area> {
        ui.label(text);
        ui.add_space(8.0);

        let id = SelectIds::SETUP_AREA;
        let options = Area::all()
            .into_iter()
            .map(|area| (area.clone(), area.name()))
            .collect();
        select_with_options(ui, id, selected, options)
    }

    fn render_player_amount(ui: &mut Ui, state: &AppState, texts: &SetupTexts) -> SetupViewResult {
        let mut result = SetupViewResult::default();
        ui.label(&texts.player_count);
        let mut new_count = state.setup_state().player_count;
        if ui.add(Slider::new(&mut new_count, 4..=18)).changed() {
            result.player_count = Some(new_count);
        }
        result
    }

    fn render_player(
        ui: &mut Ui,
        state: &AppState,
        texts: &SetupTexts,
        i: usize,
        available_colors: &[Color],
    ) -> SetupViewResult {
        let mut result = SetupViewResult::default();

        ui.horizontal(|ui| {
            ui.label(&texts.player_name);
            ui.add_space(4.0);

            let player = &state.setup_state().players[i].clone();
            let mut name = player.name.clone();
            if ui.text_edit_singleline(&mut name).changed() {
                result.update_player = Some((player.id, name, player.color.clone()));
            }

            ui.add_space(8.0);

            let selected_color = player.color.clone();

            // 色プレビューのサイズと位置を調整
            let (rect, _response) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, selected_color.to_egui_color());

            ui.add_space(4.0);

            let id = format!("{}_{}", SelectIds::SETUP_PLAYER_COLOR, player.id);
            let selected_text = selected_color.name();
            select_with_contents(ui, id, selected_text, |ui| {
                for color in available_colors {
                    let text = RichText::new(color.name())
                        .background_color(color.to_egui_color())
                        .color(choose_text_color(color.to_egui_color()));
                    if ui
                        .selectable_label(selected_color == *color, text)
                        .clicked()
                    {
                        result.update_player =
                            Some((player.id, player.name.clone(), color.clone()));
                    }
                }
            });
        });
        ui.add_space(4.0);

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
