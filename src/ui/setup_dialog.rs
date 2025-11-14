use crate::game::state::AppState;
use egui::*;

pub struct SetupDialog;

impl SetupDialog {
    pub fn show(state: &mut AppState, ctx: &egui::Context) {
        egui::Window::new("ゲーム設定")
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 600.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("エリア選択");

                    // エリア一覧（簡易実装、後で拡張可能）
                    let areas = vec![
                        ("The Skeld", "skeld"),
                        ("Mira HQ", "mira"),
                        ("Polus", "polus"),
                        ("Airship", "airship"),
                    ];

                    for (name, id) in &areas {
                        if ui.radio_value(&mut state.setup_state.selected_area_id, id.to_string(), *name).clicked() {
                            state.setup_state.selected_area_name = name.to_string();
                        }
                    }

                    ui.separator();
                    ui.heading("プレイヤー設定");

                    ui.horizontal(|ui| {
                        ui.label("プレイヤー数:");
                        ui.add(egui::Slider::new(&mut state.setup_state.player_count, 10..=15));
                    });

                    ui.separator();

                    // プレイヤー一覧の編集（簡易実装）
                    ui.label("各プレイヤーの設定:");
                    ui.label("（実装中: デフォルト設定で開始可能）");

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("ゲーム開始").clicked() {
                            state.create_game_from_setup();
                        }

                        if ui.button("キャンセル").clicked() {
                            state.cancel_setup();
                        }
                    });
                });
            });
    }
}