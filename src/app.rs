use egui::*;

use crate::constants::{AppConstants, PanelIds};
use crate::features::main::MainView;
use crate::features::player_info::PlayerInfoFeature;
use crate::features::setup_dialog::SetupDialogFeature;
use crate::i18n::keys;
use crate::state::AppState;

pub struct AmusApp {
    state: AppState,
}

impl Default for AmusApp {
    fn default() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl AmusApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        let mut app = Self { state };

        if let Some(storage) = _cc.storage {
            app.state
                .load_from_storage(Some(storage))
                .unwrap_or_default();
        }
        app
    }
}

impl eframe::App for AmusApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Err(e) = self.state.save_to_storage(Some(storage)) {
            eprintln!("Failed to save to storage: {}", e);
        }
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        // セットアップダイアログの表示
        if self.state.show_setup_dialog() {
            SetupDialogFeature::render(&mut self.state, ctx);
            return;
        }

        // 上部のメニューボタン
        self.build_menu_ui(ctx, frame);

        // メインUIの構築
        self.build_main_ui(ctx, frame);
    }
}

impl AmusApp {
    fn build_main_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 右側のプレイヤー情報パネル
        SidePanel::right(PanelIds::PLAYER_INFO)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    let head_text = RichText::new(self.state.t(keys::SIDEBAR_PLAYERS))
                        .heading()
                        .color(Color32::WHITE);
                    ui.heading(head_text);
                });
                ui.separator();
                PlayerInfoFeature::render(&mut self.state, ui);
            });

        // 残りの中央領域
        CentralPanel::default()
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                MainView::render(&mut self.state, ui);
            });
    }

    fn build_menu_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut toggle_setup = false;
        #[cfg(debug_assertions)]
        let mut toggle_debug = false;

        TopBottomPanel::top(PanelIds::MENU)
            .resizable(false)
            .default_height(50.0)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // リセットボタン
                        if ui.button("🔄 リセット").clicked() {
                            toggle_setup = true;
                        }

                        // デバッグボタン
                        #[cfg(debug_assertions)]
                        if ui.button("🐛 デバッグ").clicked() {
                            toggle_debug = true;
                        }
                    });
                });
            });

        if toggle_setup {
            self.state.toggle_setup_dialog();
        }
        #[cfg(debug_assertions)]
        if toggle_debug {
            self.state.toggle_debug_view();
        }
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::WINDOW_BG_COLOR_DARK)
            .stroke(Stroke::NONE)
    }
}
