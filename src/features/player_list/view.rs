use crate::{components::player_rect, state::AppState};
use egui::*;

pub struct PlayerListView;

impl PlayerListView {
    pub fn show(state: &mut AppState, ui: &mut Ui) {
        // 利用可能なエリア全体を取得
        let available_rect = ui.available_rect_before_wrap();

        // 横スクロール可能なエリアで中央に配置
        ScrollArea::horizontal()
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                // プレイヤー数に基づいてコンテンツ幅を計算
                let players = match state.players() {
                    Some(players) => players,
                    None => return,
                };

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

                    for player in players.iter() {
                        ui.vertical(|ui| {
                            let is_selected = state.selected_player_id() == Some(player.id);
                            let is_dragging = state.dragging_player_id() == Some(player.id);

                            let res = player_rect(ui, player, is_selected, is_dragging);

                            // クリックまたはドラッグ開始時にユーザーを選択
                            if res.clicked || res.drag_started {
                                state.set_selected_player_id(Some(player.id));
                                state.set_erase_mode(false);
                                if res.drag_started {
                                    state.set_dragging_player_id(Some(player.id));
                                }
                            }

                            // ドラッグ終了時にクリア（選択状態は保持）
                            if res.drag_stopped {
                                state.set_dragging_player_id(None);
                            }
                        });
                        ui.add_space(6.0); // プレイヤー間のスペース
                    }
                });
            });
    }
}
