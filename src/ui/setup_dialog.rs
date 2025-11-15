use super::common_texts::CommonTexts;
use crate::game::state::AppState;
use crate::i18n::keys::*;
use egui::*;

pub struct SetupDialog;

impl SetupDialog {
    pub fn show(state: &mut AppState, ctx: &egui::Context) {
        let texts = SetupTexts::get(state);
        let common = CommonTexts::get(state);

        egui::Window::new(&texts.title)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 600.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading(&texts.area_selection);

                    // エリア一覧（簡易実装、後で拡張可能）
                    let areas = vec![
                        ("The Skeld", "skeld"),
                        ("Mira HQ", "mira"),
                        ("Polus", "polus"),
                        ("Airship", "airship"),
                    ];

                    for (name, id) in &areas {
                        if ui
                            .radio_value(
                                &mut state.setup_state.selected_area_id,
                                id.to_string(),
                                *name,
                            )
                            .clicked()
                        {
                            state.setup_state.selected_area_name = name.to_string();
                        }
                    }

                    ui.separator();
                    ui.heading(&texts.player_settings);

                    ui.horizontal(|ui| {
                        ui.label(&texts.player_count);
                        ui.add(egui::Slider::new(
                            &mut state.setup_state.player_count,
                            10..=15,
                        ));
                    });

                    ui.separator();

                    // プレイヤー一覧の編集（簡易実装）
                    ui.label(&texts.player_config);
                    ui.label(&texts.config_note);

                    ui.separator();

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
    }
}

// セットアップダイアログ固有のテキスト
struct SetupTexts {
    pub title: String,
    pub area_selection: String,
    pub player_settings: String,
    pub player_count: String,
    pub player_config: String,
    pub config_note: String,
}

impl SetupTexts {
    fn get(state: &AppState) -> Self {
        Self {
            title: state.t(SETUP_TITLE).to_string(),
            area_selection: state.t(SETUP_AREA_SELECTION).to_string(),
            player_settings: state.t(SETUP_PLAYER_SETTINGS).to_string(),
            player_count: state.t(SETUP_PLAYER_COUNT).to_string(),
            player_config: state.t(SETUP_PLAYER_CONFIG).to_string(),
            config_note: state.t(SETUP_CONFIG_NOTE).to_string(),
        }
    }
}
