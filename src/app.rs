use crate::assets::AssetManager;
use crate::features::main::MainView;
use crate::features::setup_dialog::SetupView;
use crate::features::user_list::UserListView;
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        // アセットマネージャーの初期化（一度だけ実行）
        if self.state.asset_manager.is_none() {
            let asset_manager = AssetManager::new(ctx);
            self.state.asset_manager = Some(asset_manager);
        }

        // セットアップダイアログの表示
        if self.state.show_setup_dialog {
            SetupView::show(&mut self.state, ctx);
            return;
        }

        // メインUIの構築
        self.build_main_ui(ctx, frame);
    }
}

impl AmusApp {
    fn build_main_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 全画面のメインコンテンツエリア
        egui::CentralPanel::default().show(ctx, |ui| {
            MainView::render(&mut self.state, ui);
        });

        // 下部のユーザー一覧パネル
        egui::TopBottomPanel::bottom("user_list_panel")
            .resizable(false)
            .default_height(120.0)
            .frame(
                egui::Frame::none().stroke(egui::Stroke::NONE), // 枠線を完全に削除
            )
            .show(ctx, |ui| {
                UserListView::show(&mut self.state, ui);
            });

        // 上部のメニューボタン
        egui::TopBottomPanel::top("menu_button_panel")
            .resizable(false)
            .default_height(50.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 左側：メニューボタン
                    if ui.button("📋 メニュー").clicked() {
                        self.state.show_turn_menu = !self.state.show_turn_menu;
                    }

                    // 中央：デバッグボタン
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 右側：リセットボタン
                        if ui.button("🔄 リセット").clicked() {
                            self.state.show_setup_dialog = true;
                            self.state.show_turn_menu = false; // メニューを閉じる
                        }

                        // 中央：デバッグボタン（スペースで中央に配置）
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                if ui.button("🐛 デバッグ").clicked() {
                                    self.state.show_debug_view = !self.state.show_debug_view;
                                }
                            },
                        );
                    });
                });
            });

        // ターンメニューのオーバーレイ表示
        if self.state.show_turn_menu {
            self.show_turn_menu_overlay(ctx);
        }
    }

    fn show_turn_menu_overlay(&mut self, ctx: &egui::Context) {
        // 背景の半透明オーバーレイ
        let screen_rect = ctx.screen_rect();

        egui::Area::new("turn_menu_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                // 全画面の半透明背景
                ui.allocate_ui(screen_rect.size(), |ui| {
                    let painter = ui.painter();
                    painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(128));

                    // 4x4のターンボタングリッドを中央に配置
                    let grid_size = 280.0;
                    let center_pos =
                        screen_rect.center() - egui::vec2(grid_size / 2.0, grid_size / 2.0);

                    ui.allocate_ui_at_rect(
                        egui::Rect::from_min_size(center_pos, egui::vec2(grid_size, grid_size)),
                        |ui| {
                            ui.visuals_mut().panel_fill = egui::Color32::from_gray(240);

                            egui::Frame::popup(&ui.style())
                                .inner_margin(egui::Margin::same(20.0))
                                .show(ui, |ui| {
                                    ui.heading("ターン選択");
                                    ui.separator();

                                    // ターンボタングリッド
                                    if let Some(game) = &self.state.game {
                                        let total_waves = game.waves.len();
                                        egui::Grid::new("turn_grid")
                                            .num_columns(4)
                                            .spacing([10.0, 10.0])
                                            .show(ui, |ui| {
                                                for turn_index in 0..total_waves {
                                                    let turn_number = turn_index + 1;
                                                    let is_current =
                                                        self.state.current_wave_index == turn_index;

                                                    let button_text = if is_current {
                                                        format!("ターン {} (現在)", turn_number)
                                                    } else {
                                                        format!("ターン {}", turn_number)
                                                    };

                                                    let button = ui.button(button_text);
                                                    if button.clicked() {
                                                        // ターンを選択して該当waveに切り替え
                                                        self.state.select_wave(turn_index);
                                                        self.state.show_turn_menu = false;
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
                        },
                    );

                    // オーバーレイ外のクリックを検知
                    let response = ui.allocate_response(screen_rect.size(), egui::Sense::click());
                    if response.clicked() {
                        let click_pos = response.interact_pointer_pos().unwrap_or_default();
                        let grid_rect =
                            egui::Rect::from_min_size(center_pos, egui::vec2(grid_size, grid_size));

                        // グリッド外がクリックされた場合、メニューを閉じる
                        if !grid_rect.contains(click_pos) {
                            self.state.show_turn_menu = false;
                        }
                    }
                });
            });
    }
}
