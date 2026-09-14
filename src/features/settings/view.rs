use egui::*;

use crate::common::CommonTexts;
use crate::components::select_with_options;
use crate::constants::{AppConstants, SelectIds};
use crate::i18n::keys::*;
use crate::state::{SettingsAction, SettingsTab, Slices};

/// UI 拡大率のプルダウン選択肢（有効範囲内のプリセット）。
const UI_SCALE_PRESETS: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0];

pub struct SettingsView;

impl SettingsView {
    pub fn render(slices: &Slices<'_>, ctx: &Context) -> Vec<SettingsAction> {
        let texts = SettingsTexts::get(slices);
        let common = CommonTexts::get(slices);
        let current_tab = slices.ui().settings_tab();

        let mut actions = Vec::new();

        const WINDOW_SIZE: Vec2 = vec2(480.0, 320.0);

        Window::new(&texts.title)
            .collapsible(false)
            .fixed_size(WINDOW_SIZE)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // fixed_size は初期レイアウトの上限になるだけで、内容が小さいタブでは
                // ウィンドウ自体が縮んでしまう（egui の Window は resizable=false かつ
                // with_stroke=false だと最終サイズを内容に合わせて決める）。
                // 内容側の最小サイズを固定してタブ間で高さが変わらないようにする。
                ui.set_min_size(WINDOW_SIZE);

                // 閉じるボタンをタブ内容量に関係なく常にウィンドウ下端に固定するため、
                // 上部（タブ+内容）と下部（閉じるボタン）の領域をあらかじめ分けて描画する。
                const FOOTER_HEIGHT: f32 = 44.0;
                let full_rect = ui.max_rect();
                let mut content_rect = full_rect;
                content_rect.max.y -= FOOTER_HEIGHT;
                let mut footer_rect = full_rect;
                footer_rect.min.y = full_rect.max.y - FOOTER_HEIGHT;

                ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
                    ui.horizontal(|ui| {
                        for (tab, label) in [
                            (SettingsTab::Game, &texts.tab_game),
                            (SettingsTab::Display, &texts.tab_display),
                            (SettingsTab::Voice, &texts.tab_voice),
                            (SettingsTab::App, &texts.tab_app),
                        ] {
                            if ui
                                .selectable_label(current_tab == tab, label.as_str())
                                .clicked()
                            {
                                actions.push(SettingsAction::SelectTab(tab));
                            }
                        }
                    });
                    ui.separator();
                    ui.add_space(8.0);

                    match current_tab {
                        SettingsTab::Game => {
                            Self::render_game_tab(slices, &texts, ui, &mut actions)
                        }
                        SettingsTab::Display => {
                            Self::render_display_tab(slices, &texts, ui, &mut actions)
                        }
                        SettingsTab::Voice => Self::render_voice_tab(&texts, ui),
                        SettingsTab::App => Self::render_app_tab(&texts, ui, &mut actions),
                    }
                });

                ui.scope_builder(UiBuilder::new().max_rect(footer_rect), |ui| {
                    ui.separator();
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        if ui.button(&common.button_close).clicked() {
                            actions.push(SettingsAction::Close);
                        }
                    });
                });
            });

        actions
    }

    fn render_game_tab(
        slices: &Slices<'_>,
        texts: &SettingsTexts,
        ui: &mut Ui,
        actions: &mut Vec<SettingsAction>,
    ) {
        ui.horizontal(|ui| {
            ui.label(&texts.route_line_width);
            let mut width = slices.ui().route_line_width();
            if ui
                .add(Slider::new(
                    &mut width,
                    AppConstants::ROUTE_LINE_WIDTH_MIN..=AppConstants::ROUTE_LINE_WIDTH_MAX,
                ))
                .changed()
            {
                actions.push(SettingsAction::SetRouteLineWidth(width));
            }
        });
    }

    fn render_display_tab(
        slices: &Slices<'_>,
        texts: &SettingsTexts,
        ui: &mut Ui,
        actions: &mut Vec<SettingsAction>,
    ) {
        ui.horizontal(|ui| {
            ui.label(&texts.ui_scale);
            ui.add_space(8.0);

            let mut selected = slices.ui().ui_scale();
            let options: Vec<(Option<f32>, String)> =
                std::iter::once((None, texts.ui_scale_auto_label.clone()))
                    .chain(
                        UI_SCALE_PRESETS
                            .iter()
                            .map(|&scale| (Some(scale), format!("{:.0}%", scale * 100.0))),
                    )
                    .collect();

            if let Some(choice) =
                select_with_options(ui, SelectIds::SETTINGS_UI_SCALE, &mut selected, options)
            {
                match choice {
                    Some(scale) => actions.push(SettingsAction::SetUiScale(scale)),
                    None => actions.push(SettingsAction::ResetUiScale),
                }
            }
        });
    }

    fn render_voice_tab(texts: &SettingsTexts, ui: &mut Ui) {
        ui.label(&texts.voice_placeholder);
    }

    #[allow(unused_variables)]
    fn render_app_tab(texts: &SettingsTexts, ui: &mut Ui, actions: &mut Vec<SettingsAction>) {
        ui.horizontal(|ui| {
            ui.label(&texts.app_version);
            ui.label(concat!("v", env!("CARGO_PKG_VERSION")));
        });

        #[cfg(debug_assertions)]
        {
            ui.add_space(8.0);
            if ui.button(&texts.debug_toggle).clicked() {
                actions.push(SettingsAction::ToggleDebugView);
            }
        }
    }
}

struct SettingsTexts {
    title: String,
    tab_game: String,
    tab_display: String,
    tab_voice: String,
    tab_app: String,
    route_line_width: String,
    ui_scale: String,
    ui_scale_auto_label: String,
    voice_placeholder: String,
    app_version: String,
    #[cfg(debug_assertions)]
    debug_toggle: String,
}

impl SettingsTexts {
    fn get(slices: &Slices<'_>) -> Self {
        Self {
            title: slices.t(SETTINGS_TITLE),
            tab_game: slices.t(SETTINGS_TAB_GAME),
            tab_display: slices.t(SETTINGS_TAB_DISPLAY),
            tab_voice: slices.t(SETTINGS_TAB_VOICE),
            tab_app: slices.t(SETTINGS_TAB_APP),
            route_line_width: slices.t(SETTINGS_ROUTE_LINE_WIDTH),
            ui_scale: slices.t(SETTINGS_UI_SCALE),
            ui_scale_auto_label: slices.t(SETTINGS_UI_SCALE_AUTO),
            voice_placeholder: slices.t(SETTINGS_VOICE_PLACEHOLDER),
            app_version: slices.t(SETTINGS_APP_VERSION),
            #[cfg(debug_assertions)]
            debug_toggle: slices.t(SETTINGS_DEBUG_TOGGLE),
        }
    }
}
