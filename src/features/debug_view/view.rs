use crate::state::AppState;

pub struct DebugView;

impl DebugView {
    pub fn render(state: &AppState, render_flag: bool, ctx: &egui::Context) {
        if !render_flag {
            return;
        }

        let screen_rect = ctx.content_rect();
        let window_width = 300.0; // 固定幅
        let window_pos = egui::pos2(
            screen_rect.center().x - window_width / 2.0,
            screen_rect.min.y + 60.0, // メニューバーの下に配置
        );

        egui::Area::new(egui::Id::new("debug_view"))
            .fixed_pos(window_pos)
            .show(ctx, |ui| {
                Self::show_content(state, ui, ctx, window_width);
            });
    }

    fn show_content(state: &AppState, ui: &mut egui::Ui, ctx: &egui::Context, window_width: f32) {
        egui::Frame::popup(ui.style())
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_width(window_width);
                ui.heading("🐛 デバッグ情報");
                ui.separator();

                Self::show_debug_table(state, ui, ctx);
            });
    }

    fn show_debug_table(state: &AppState, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 2列のテーブル形式で表示
        egui::Grid::new("debug_grid")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // 選択中ユーザー
                ui.label("選択中ユーザー:");
                if let Some(player_id) = state.selected_player_id() {
                    if let Some(game) = state.game() {
                        if let Some(player) = game.players.get(player_id) {
                            ui.label(format!("{} (player_id={})", player.name, player_id));
                        } else {
                            ui.label(format!("player_id={} (存在しない)", player_id));
                        }
                    } else {
                        ui.label("ゲームが存在しません");
                    }
                } else {
                    ui.label("なし");
                }
                ui.end_row();

                // ドラッグ中ユーザー
                ui.label("ドラッグ中ユーザー:");
                if let Some(player_id) = state.dragging_player_id() {
                    ui.label(format!("player_id={}", player_id));
                } else {
                    ui.label("なし");
                }
                ui.end_row();

                // ドラッグ中ポイント
                ui.label("ドラッグ中ポイント:");
                if let Some(pointer_pos) = ctx.pointer_latest_pos() {
                    ui.label(format!("x={:.1}, y={:.1}", pointer_pos.x, pointer_pos.y));
                } else {
                    ui.label("なし");
                }
                ui.end_row();
            });
    }
}
