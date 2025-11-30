use crate::state::AppState;
use egui::*;

pub struct UserListView;

impl UserListView {
    pub fn show(state: &mut AppState, ui: &mut Ui) {
        // 利用可能なエリア全体を取得
        let available_rect = ui.available_rect_before_wrap();

        // 横スクロール可能なエリアで中央に配置
        ScrollArea::horizontal()
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                // プレイヤー数に基づいてコンテンツ幅を計算
                let players = state.setup_state().players.clone();
                let player_width = 50.0; // 各プレイヤーの幅
                let total_content_width = players.len() as f32 * player_width;
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

                    for (user_id, user) in players.iter().enumerate() {
                        ui.vertical(|ui| {
                            // ユーザー色の矩形（30x30px）
                            let rect_size = Vec2::new(30.0, 30.0);
                            let (rect, response) =
                                ui.allocate_exact_size(rect_size, Sense::click_and_drag());

                            // クリックまたはドラッグ開始時にユーザーを選択
                            if response.clicked() || response.drag_started() {
                                state.set_selected_user_id(Some(user_id));
                                state.set_erase_mode(false);
                                if response.drag_started() {
                                    state.set_dragging_user_id(Some(user_id));
                                }
                            }

                            // ドラッグ終了時にクリア（選択状態は保持）
                            if response.drag_stopped() {
                                state.set_dragging_user_id(None);
                            }

                            // ユーザーの色を取得
                            let user_color = if let Some(player_config) = state
                                .setup_state()
                                .players
                                .iter()
                                .find(|p| p.name == user.name)
                            {
                                player_config.color.to_egui_color()
                            } else {
                                Color32::GRAY
                            };

                            // 選択中またはドラッグ中の視覚的フィードバック
                            let is_selected = state.selected_user_id() == Some(user_id);
                            let is_dragging = state.dragging_user_id() == Some(user_id);
                            let color = if is_dragging || (is_selected && response.hovered()) {
                                user_color.linear_multiply(0.7) // 少し暗くする
                            } else if is_selected {
                                user_color.linear_multiply(0.9) // 選択中は少し暗く
                            } else {
                                user_color
                            };

                            // 矩形を描画
                            ui.painter().rect_filled(rect, 4.0, color);

                            // 選択中のユーザーには枠線を表示
                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    (2.0, Color32::WHITE),
                                    StrokeKind::Inside,
                                );
                            }
                        });
                        ui.add_space(6.0); // プレイヤー間のスペース
                    }
                });
            });
    }
}
