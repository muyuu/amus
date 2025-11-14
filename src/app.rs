use crate::game::state::AppState;
use crate::ui::{sidebar::Sidebar, setup_dialog::SetupDialog, main_content::MainContent};

pub struct AmusApp {
    state: AppState,
}

impl Default for AmusApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for AmusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // メインメニューバー
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("新規ゲーム").clicked() {
                        self.state.start_new_game();
                        ui.close_menu();
                    }
                    if ui.button("ゲームを読み込む").clicked() {
                        // TODO: ファイル読み込み
                        ui.close_menu();
                    }
                    if ui.button("ゲームを保存").clicked() {
                        // TODO: ファイル保存
                        ui.close_menu();
                    }
                });
            });
        });

        // 左サイドバー（ターン管理とプレイヤー選択）
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                Sidebar::show(&mut self.state, ui);
            });

        // メインコンテンツエリア
        egui::CentralPanel::default().show(ctx, |ui| {
            MainContent::show(&mut self.state, ui);
        });

        // ゲーム設定ダイアログ
        if self.state.show_setup_dialog {
            SetupDialog::show(&mut self.state, ctx);
        }
    }
}

