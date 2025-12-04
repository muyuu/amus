use egui::*;

use crate::assets::AssetManager;
use crate::constants::AppConstants;
use crate::features::main::MainView;
use crate::features::player_info::PlayerInfoFeature;
use crate::features::player_list::PlayerListView;
use crate::features::setup_dialog::SetupView;
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

impl eframe::App for AmusApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        // アセットマネージャーの初期化（一度だけ実行）
        if self.state.asset_manager().is_none() {
            let asset_manager = AssetManager::new(ctx);
            self.state.set_asset_manager(asset_manager);
        }

        // セットアップダイアログの表示
        if self.state.show_setup_dialog() {
            SetupView::show(&mut self.state, ctx);
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
        SidePanel::right("player_info_panel")
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                let head_text = RichText::new(self.state.t(keys::SIDEBAR_PLAYERS))
                    .heading()
                    .color(Color32::WHITE);
                ui.heading(head_text);
                ui.separator();
                PlayerInfoFeature::render(&mut self.state, ui);
            });

        // 全画面のメインコンテンツエリア
        CentralPanel::default()
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                MainView::render(&mut self.state, ui);
            });

        // 下部のユーザー一覧パネル
        TopBottomPanel::bottom("player_list_panel")
            .resizable(false)
            .default_height(90.0)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                PlayerListView::show(&mut self.state, ui);
            });
    }

    fn build_menu_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu_button_panel")
            .resizable(false)
            .default_height(50.0)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 中央：デバッグボタン
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // 右側：リセットボタン
                        if ui.button("🔄 リセット").clicked() {
                            self.state.data_mut().show_setup_dialog = true;
                        }

                        // 中央：デバッグボタン（スペースで中央に配置）
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            Layout::top_down(Align::Center),
                            |ui| {
                                if ui.button("🐛 デバッグ").clicked() {
                                    self.state.data_mut().show_debug_view =
                                        !self.state.show_debug_view();
                                }
                            },
                        );
                    });
                });
            });
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::WINDOW_BG_COLOR_DARK)
            .stroke(Stroke::NONE)
    }
}
