use egui::*;

use crate::app_action::AppAction;
use crate::constants::{AppConstants, PanelIds};
use crate::features::main::MainView;
use crate::features::player_info::PlayerInfoFeature;
use crate::features::setup_dialog::SetupDialogFeature;
use crate::features::Features;
use crate::i18n::keys;
use crate::resources::Resources;
use crate::state::{Actions, AppState};

pub struct AmusApp {
    state: AppState,
    // resources を読むのは native 専用の voice_memo（① update / ③ handle）のみ
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    resources: Resources,
    features: Features,
}

impl Default for AmusApp {
    fn default() -> Self {
        let resources = Resources::new();
        #[cfg(not(target_arch = "wasm32"))]
        let features = Features::new(&resources);
        #[cfg(target_arch = "wasm32")]
        let features = Features::new();

        Self {
            state: AppState::new(),
            resources,
            features,
        }
    }
}

impl AmusApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        let resources = Resources::new();
        #[cfg(not(target_arch = "wasm32"))]
        let features = Features::new(&resources);
        #[cfg(target_arch = "wasm32")]
        let features = Features::new();

        let mut app = Self {
            state,
            resources,
            features,
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

        // セットアップダイアログ表示中はそれだけを描画
        if self.state.show_setup_dialog() {
            let actions = SetupDialogFeature::render(&self.state, ctx);
            self.handle_actions(actions);
            return;
        }

        // ① リアルタイム更新（リソース系のステートフル feature）
        #[cfg(not(target_arch = "wasm32"))]
        self.features.voice_memo.update(&self.resources, ctx);

        // 上部のメニュー（アプリシェル）
        self.build_menu_ui(ctx, frame);

        // ② 描画して Action を収集
        let actions = self.build_main_ui(ctx, frame);

        // ③ Action を 1 箇所で dispatch
        self.handle_actions(actions);
    }
}

impl AmusApp {
    fn build_main_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) -> Vec<AppAction> {
        // self の disjoint なフィールドを先に分けて借りておく（クロージャが self 全体を掴まないように）
        let state = &self.state;
        let features = &mut self.features;
        let mut actions = Vec::new();

        // 右側のプレイヤー情報パネル
        let side = SidePanel::right(PanelIds::PLAYER_INFO)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    let head_text = RichText::new(state.t(keys::SIDEBAR_PLAYERS))
                        .heading()
                        .color(Color32::WHITE);
                    ui.heading(head_text);
                });
                ui.separator();
                PlayerInfoFeature::render(state, ui)
            });
        actions.extend(side.inner);

        // 残りの中央領域
        let central = CentralPanel::default()
            .frame(Self::get_frame())
            .show(ctx, |ui| MainView::render(state, features, ui));
        actions.extend(central.inner);

        actions
    }

    /// 全 Feature が返した Action を 1 箇所で dispatch する（中央 dispatch の③）。
    fn handle_actions(&mut self, actions: Vec<AppAction>) {
        for action in actions {
            match action {
                AppAction::Game(a) => Actions::new(&mut self.state).handle_game(a),
                AppAction::Player(a) => Actions::new(&mut self.state).handle_player(a),
                AppAction::PlayerInfo(a) => Actions::new(&mut self.state).handle_player_info(a),
                AppAction::Location(a) => Actions::new(&mut self.state).handle_location(a),
                AppAction::RouteDrawing(a) => Actions::new(&mut self.state).handle_route_drawing(a),
                AppAction::Eraser(a) => Actions::new(&mut self.state).handle_eraser(a),
                AppAction::Setup(a) => Actions::new(&mut self.state).handle_setup(a),
                AppAction::Window(a) => Actions::new(&mut self.state).handle_window(a),
                #[cfg(not(target_arch = "wasm32"))]
                AppAction::VoiceMemo(a) => self
                    .features
                    .voice_memo
                    .handle_action(&mut self.resources, a),
            }
        }
    }

    fn build_menu_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut toggle_setup = false;
        #[cfg(debug_assertions)]
        let mut toggle_debug = false;

        TopBottomPanel::top(PanelIds::MENU)
            .resizable(false)
            .default_height(50.0)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 左: アプリのバージョン（ビルド時の Cargo.toml の値）
                    ui.weak(concat!("v", env!("CARGO_PKG_VERSION")));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // リセットボタン
                        if ui.button("🔄 リセット").clicked() {
                            toggle_setup = true;
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
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::WINDOW_BG_COLOR_DARK)
            .stroke(Stroke::NONE)
    }
}
