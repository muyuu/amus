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
        TopBottomPanel::top("menu_button_panel")
            .resizable(false)
            .default_height(50.0)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 左側：メニューボタン
                    if ui.button("📋 メニュー").clicked() {
                        self.state.set_show_turn_menu(!self.state.show_turn_menu());
                    }

                    // 中央：デバッグボタン
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // 右側：リセットボタン
                        if ui.button("🔄 リセット").clicked() {
                            self.state.data_mut().show_setup_dialog = true;
                            self.state.data_mut().show_turn_menu = false; // メニューを閉じる
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

        // ターンメニューのオーバーレイ表示
        if self.state.show_turn_menu() {
            self.show_turn_menu_overlay(ctx);
        }

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

    fn show_turn_menu_overlay(&mut self, ctx: &Context) {
        // 背景の半透明オーバーレイ
        let screen_rect = ctx.content_rect();

        Area::new("turn_menu_overlay".into())
            .fixed_pos(pos2(0.0, 0.0))
            .show(ctx, |ui| {
                // 全画面の半透明背景
                ui.allocate_ui(screen_rect.size(), |ui| {
                    let painter = ui.painter();
                    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(128));

                    // 4x4のターンボタングリッドを中央に配置
                    let grid_size = 280.0;
                    let center_pos = screen_rect.center() - vec2(grid_size / 2.0, grid_size / 2.0);

                    let max_rect = Rect::from_min_size(center_pos, vec2(grid_size, grid_size));
                    ui.scope_builder(UiBuilder::new().max_rect(max_rect), |ui| {
                        ui.visuals_mut().panel_fill = Color32::from_gray(240);

                        Frame::popup(ui.style())
                            .inner_margin(Margin::same(20))
                            .show(ui, |ui| {
                                ui.heading("ターン選択");
                                ui.separator();

                                // ターンボタングリッド
                                if let Some(game) = &self.state.game() {
                                    let total_waves = game.waves.len();
                                    Grid::new("turn_grid")
                                        .num_columns(4)
                                        .spacing([10.0, 10.0])
                                        .show(ui, |ui| {
                                            for turn_index in 0..total_waves {
                                                let turn_number = turn_index + 1;
                                                let is_current =
                                                    self.state.current_wave_index() == turn_index;

                                                let button_text = if is_current {
                                                    format!("ターン {} (現在)", turn_number)
                                                } else {
                                                    format!("ターン {}", turn_number)
                                                };

                                                let button = ui.button(button_text);
                                                if button.clicked() {
                                                    // ターンを選択して該当waveに切り替え
                                                    self.state.select_wave(turn_index);
                                                    self.state.set_show_turn_menu(false);
                                                }

                                                // 4列で改行
                                                if (turn_index + 1) % 4 == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });
                                } else {
                                    ui.label("ゲームが開始されていません");
                                }
                            });
                    });

                    // オーバーレイ外のクリックを検知
                    let response = ui.allocate_response(screen_rect.size(), Sense::click());
                    if response.clicked() {
                        let click_pos = response.interact_pointer_pos().unwrap_or_default();
                        let grid_rect = Rect::from_min_size(center_pos, vec2(grid_size, grid_size));

                        // グリッド外がクリックされた場合、メニューを閉じる
                        if !grid_rect.contains(click_pos) {
                            self.state.data_mut().show_turn_menu = false;
                        }
                    }
                });
            });
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::WINDOW_BG_COLOR_DARK)
            .stroke(Stroke::NONE)
    }
}
