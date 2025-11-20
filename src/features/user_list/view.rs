use crate::models::Game;
use crate::state::AppState;

pub struct UserListView;

impl UserListView {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        if let Some(game) = &state.game {
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
                                let user_color = if let Some(player_config) = state
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
                                    state.selected_user_id = Some(user_id);
                                    if response.drag_started() {
                                        state.dragging_user_id = Some(user_id);
                                    }
                                }

                                // ドラッグ終了時にクリア（選択状態は保持）
                                if response.drag_stopped() {
                                    state.dragging_user_id = None;
                                }

                                // 選択中またはドラッグ中の視覚的フィードバック
                                let is_selected = state.selected_user_id == Some(user_id);
                                let is_dragging = state.dragging_user_id == Some(user_id);
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
}
