use egui::*;

use crate::constants::{AppConstants, PanelIds};
use crate::features::main::MainView;
use crate::features::player_info::PlayerInfoFeature;
use crate::features::setup_dialog::SetupDialogFeature;
use crate::i18n::keys;
use crate::state::AppState;

#[cfg(not(target_arch = "wasm32"))]
use crate::features::voice_memo::VoiceMemoFeature;

pub struct AmusApp {
    state: AppState,
    #[cfg(not(target_arch = "wasm32"))]
    voice_memo: VoiceMemoFeature,
    #[cfg(not(target_arch = "wasm32"))]
    show_voice_memo: bool,
}

impl Default for AmusApp {
    fn default() -> Self {
        Self {
            state: AppState::new(),
            #[cfg(not(target_arch = "wasm32"))]
            voice_memo: VoiceMemoFeature::new(),
            #[cfg(not(target_arch = "wasm32"))]
            show_voice_memo: false,
        }
    }
}

impl AmusApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        let mut app = Self {
            state,
            #[cfg(not(target_arch = "wasm32"))]
            voice_memo: VoiceMemoFeature::new(),
            #[cfg(not(target_arch = "wasm32"))]
            show_voice_memo: false,
        };

        if let Some(storage) = _cc.storage {
            app.state
                .load_from_storage(Some(storage))
                .unwrap_or_default();
        }
        app
    }
}

impl eframe::App for AmusApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Err(e) = self.state.save_to_storage(Some(storage)) {
            eprintln!("Failed to save to storage: {}", e);
        }
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        // セットアップダイアログの表示
        if self.state.show_setup_dialog() {
            SetupDialogFeature::render(&mut self.state, ctx);
            return;
        }

        // 上部のメニューボタン
        self.build_menu_ui(ctx, frame);

        // メインUIの構築
        self.build_main_ui(ctx, frame);

        // 音声メモウィンドウ（ネイティブのみ）
        #[cfg(not(target_arch = "wasm32"))]
        self.build_voice_memo_window(ctx);
    }
}

impl AmusApp {
    fn build_main_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 右側のプレイヤー情報パネル
        SidePanel::right(PanelIds::PLAYER_INFO)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    let head_text = RichText::new(self.state.t(keys::SIDEBAR_PLAYERS))
                        .heading()
                        .color(Color32::WHITE);
                    ui.heading(head_text);
                });
                ui.separator();
                PlayerInfoFeature::render(&mut self.state, ui);
            });

        // 残りの中央領域
        CentralPanel::default()
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                MainView::render(&mut self.state, ui);
            });
    }

    fn build_menu_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut toggle_setup = false;
        let mut toggle_debug = false;
        #[cfg(not(target_arch = "wasm32"))]
        let mut toggle_voice_memo = false;

        TopBottomPanel::top(PanelIds::MENU)
            .resizable(false)
            .default_height(50.0)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // リセットボタン
                        if ui.button("🔄 リセット").clicked() {
                            toggle_setup = true;
                        }

                        // 音声メモボタン（ネイティブのみ）
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("🎤 音声メモ").clicked() {
                            toggle_voice_memo = true;
                        }

                        // デバッグボタン
                        #[cfg(debug_assertions)]
                        if ui.button("🐛 デバッグ").clicked() {
                            toggle_debug = true;
                        }
                    });
                });
            });

        if toggle_setup {
            self.state.toggle_setup_dialog();
        }
        #[cfg(debug_assertions)]
        if toggle_debug {
            self.state.toggle_debug_view();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if toggle_voice_memo {
            self.show_voice_memo = !self.show_voice_memo;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn build_voice_memo_window(&mut self, ctx: &Context) {
        if !self.show_voice_memo {
            return;
        }

        // プレイヤー情報をコンテキストとして設定
        self.update_voice_memo_context();

        Window::new("音声メモ")
            .collapsible(true)
            .resizable(true)
            .default_size([300.0, 400.0])
            .show(ctx, |ui| {
                self.voice_memo.render(ui, ctx);
            });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_voice_memo_context(&mut self) {
        if let Some(players) = self.state.players() {
            let player_info: Vec<(String, String)> = players
                .iter()
                .map(|p| (p.name.clone(), p.color.name_ja().to_string()))
                .collect();
            self.voice_memo.set_context(&player_info);
        }
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::WINDOW_BG_COLOR_DARK)
            .stroke(Stroke::NONE)
    }
}
