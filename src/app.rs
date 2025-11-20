use crate::assets::AssetManager;
use crate::state::AppState;
use crate::ui::{main_content::MainContent, setup_dialog::SetupDialog};

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
        // アセットマネージャーの初期化（一度だけ実行）
        if self.state.asset_manager.is_none() {
            let asset_manager = AssetManager::new(ctx);
            self.state.asset_manager = Some(asset_manager);
        }

        // セットアップダイアログの表示
        if self.state.show_setup_dialog {
            SetupDialog::show(&mut self.state, ctx);
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
            MainContent::show(&mut self.state, ui);
        });

        // 下部のユーザー一覧パネル
        egui::TopBottomPanel::bottom("user_list_panel")
            .resizable(false)
            .default_height(120.0)
            .frame(
                egui::Frame::none().stroke(egui::Stroke::NONE), // 枠線を完全に削除
            )
            .show(ctx, |ui| {
                self.show_user_list(ui);
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

    fn show_user_list(&mut self, ui: &mut egui::Ui) {
        if let Some(game) = &self.state.game {
            // 利用可能なエリア全体を取得
            let available_rect = ui.available_rect_before_wrap();

            // 横スクロール可能なエリアで中央に配置
            egui::ScrollArea::horizontal()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    // プレイヤー数に基づいてコンテンツ幅を計算
                    let player_width = 50.0; // 各プレイヤーの幅（矩形40px + スペース10px）
                    let total_content_width = game.users.len() as f32 * player_width;
                    let available_width = available_rect.width();

                    // 中央揃えのためのパディングを計算
                    let padding = if total_content_width < available_width {
                        (available_width - total_content_width) / 2.0
                    } else {
                        0.0
                    };

                    ui.horizontal(|ui| {
                        // 左側にパディングを追加
                        if padding > 0.0 {
                            ui.add_space(padding);
                        }

                        for (user_id, user) in game.users.iter().enumerate() {
                            ui.vertical(|ui| {
                                // ユーザーの色を表示（setup_stateから取得）
                                let user_color = if let Some(player_config) = self
                                    .state
                                    .setup_state
                                    .players
                                    .iter()
                                    .find(|p| p.name == user.name)
                                {
                                    player_config.color.to_egui_color()
                                } else {
                                    egui::Color32::GRAY // デフォルト色
                                };

                                // カラー矩形（正方形）をクリック/ドラッグ可能にする
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::Vec2::new(40.0, 40.0),
                                    egui::Sense::click_and_drag(),
                                );

                                // クリックまたはドラッグ開始時にユーザーを選択
                                if response.clicked() || response.drag_started() {
                                    self.state.selected_user_id = Some(user_id);
                                    if response.drag_started() {
                                        self.state.dragging_user_id = Some(user_id);
                                    }
                                }

                                // ドラッグ終了時にクリア（選択状態は保持）
                                if response.drag_stopped() {
                                    self.state.dragging_user_id = None;
                                }

                                // 選択中またはドラッグ中の視覚的フィードバック
                                let is_selected = self.state.selected_user_id == Some(user_id);
                                let is_dragging = self.state.dragging_user_id == Some(user_id);
                                let color = if is_dragging || (is_selected && response.hovered()) {
                                    user_color.linear_multiply(0.7) // 少し暗くする
                                } else if is_selected {
                                    user_color.linear_multiply(0.9) // 選択中は少し暗く
                                } else {
                                    user_color
                                };

                                ui.painter().rect_filled(rect, 4.0, color);

                                // 選択中のユーザーには枠線を表示
                                if is_selected {
                                    ui.painter().rect_stroke(
                                        rect,
                                        4.0,
                                        (2.0, egui::Color32::WHITE),
                                    );
                                }

                                // ユーザー名（矩形の下に配置）
                                ui.label(&user.name);
                            });
                            ui.add_space(10.0); // プレイヤー間のスペース
                        }
                    });
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("ゲームが開始されていません");
            });
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
