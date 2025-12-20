use egui::*;

use crate::{components::grid, constants::GridIds, state::Slices};

pub struct DebugView;

impl DebugView {
    pub fn render(slices: &Slices<'_>, render_flag: bool, ctx: &Context) {
        if !render_flag {
            return;
        }

        let screen_rect = ctx.content_rect();
        let window_width = 300.0; // 固定幅
        let window_pos = pos2(
            screen_rect.center().x - window_width / 2.0,
            screen_rect.min.y + 60.0, // メニューバーの下に配置
        );

        Area::new(Id::new("debug_view"))
            .fixed_pos(window_pos)
            .show(ctx, |ui| {
                Self::render_content(slices, ui, ctx, window_width);
            });
    }

    fn render_content(slices: &Slices<'_>, ui: &mut Ui, ctx: &Context, window_width: f32) {
        Frame::popup(ui.style())
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.set_width(window_width);
                ui.heading("デバッグ情報");
                ui.separator();

                Self::render_debug_table(slices, ui, ctx);
            });
    }

    fn render_debug_table(slices: &Slices<'_>, ui: &mut Ui, ctx: &Context) {
        let player_slice = slices.player();
        let ui_slice = slices.ui();

        // 2列のテーブル形式で表示
        grid(ui, GridIds::DEBUG_TABLE, |ui| {
            // 選択中ユーザー
            ui.label("選択中ユーザー:");
            if let Some(player_id) = player_slice.selected_player_id() {
                if let Some(player) = player_slice.player(player_id) {
                    ui.label(format!("{} (player_id={:?})", player.name, player_id));
                } else {
                    ui.label(format!("player_id={:?} (存在しない)", player_id));
                }
            } else {
                ui.label("なし");
            }
            ui.end_row();

            // ドラッグ中ユーザー
            ui.label("ドラッグ中ユーザー:");
            if let Some(player_id) = ui_slice.dragging_player_id() {
                ui.label(format!("player_id={:?}", player_id));
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
