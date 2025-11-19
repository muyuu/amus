pub struct DebugView;

impl DebugView {
    pub fn show(ctx: &egui::Context, dragging_user_id: Option<usize>) {
        let screen_rect = ctx.screen_rect();
        let window_width = 300.0; // 固定幅
        let window_pos = egui::pos2(
            screen_rect.center().x - window_width / 2.0,
            screen_rect.min.y + 60.0, // メニューバーの下に配置
        );

        egui::Area::new(egui::Id::new("debug_view"))
            .fixed_pos(window_pos)
            .show(ctx, |ui| {
                Self::show_content(ui, ctx, dragging_user_id, window_width);
            });
    }

    fn show_content(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        dragging_user_id: Option<usize>,
        window_width: f32,
    ) {
        egui::Frame::popup(&ui.style())
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.set_width(window_width);
                ui.heading("🐛 デバッグ情報");
                ui.separator();

                Self::show_debug_table(ui, ctx, dragging_user_id);
            });
    }

    fn show_debug_table(ui: &mut egui::Ui, ctx: &egui::Context, dragging_user_id: Option<usize>) {
        // 2列のテーブル形式で表示
        egui::Grid::new("debug_grid")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // ドラッグ中ユーザー
                ui.label("ドラッグ中ユーザー:");
                if let Some(user_id) = dragging_user_id {
                    ui.label(format!("user_id={}", user_id));
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
