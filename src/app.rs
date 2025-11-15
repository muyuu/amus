use crate::game::state::AppState;
use crate::i18n::{keys::*, Language};
use crate::ui::{main_content::MainContent, setup_dialog::SetupDialog, sidebar::Sidebar};

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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // メインメニューバー
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // アプリ固有のテキスト
                let texts = AppTexts::get(&self.state);

                ui.menu_button(&texts.file_menu, |ui| {
                    if ui.button(&texts.new_game).clicked() {
                        self.state.start_new_game();
                        ui.close_menu();
                    }
                    if ui.button(&texts.load_game).clicked() {
                        // TODO: ファイル読み込み
                        ui.close_menu();
                    }
                    if ui.button(&texts.save_game).clicked() {
                        // TODO: ファイル保存
                        ui.close_menu();
                    }
                });

                ui.menu_button(&texts.language_menu, |ui| {
                    let current_lang = self.state.current_language();
                    for lang in Language::all() {
                        if ui
                            .selectable_label(current_lang == lang, lang.name())
                            .clicked()
                        {
                            self.state.set_language(lang);
                            ui.close_menu();
                        }
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

// アプリケーション固有のテキスト（メニューバー等）
struct AppTexts {
    pub file_menu: String,
    pub new_game: String,
    pub load_game: String,
    pub save_game: String,
    pub language_menu: String,
}

impl AppTexts {
    fn get(state: &AppState) -> Self {
        Self {
            file_menu: state.t(MENU_FILE).to_string(),
            new_game: state.t(MENU_NEW_GAME).to_string(),
            load_game: state.t(MENU_LOAD_GAME).to_string(),
            save_game: state.t(MENU_SAVE_GAME).to_string(),
            language_menu: state.t(MENU_LANGUAGE).to_string(),
        }
    }
}
