use egui::*;

use crate::app_action::AppAction;
use crate::constants::{AppConstants, PanelIds};
use crate::features::main::MainView;
use crate::features::player_info::PlayerInfoFeature;
use crate::features::setup_dialog::SetupDialogFeature;
use crate::features::StatefulFeatures;
use crate::i18n::keys;
use crate::resources::Resources;
use crate::state::{Actions, AppState};

pub struct AmusApp {
    state: AppState,
    // resources を読むのは native 専用の voice_memo（① update / ③ handle）のみ
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    resources: Resources,
    features: StatefulFeatures,
}

impl Default for AmusApp {
    fn default() -> Self {
        let resources = Resources::new();
        #[cfg(not(target_arch = "wasm32"))]
        let features = StatefulFeatures::new(&resources);
        #[cfg(target_arch = "wasm32")]
        let features = StatefulFeatures::new();

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
        let features = StatefulFeatures::new(&resources);
        #[cfg(target_arch = "wasm32")]
        let features = StatefulFeatures::new();

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
        // ユーザー指定があればその倍率、なければネイティブ DPI に追従する。
        let ppp = self
            .state
            .ui_scale()
            .unwrap_or_else(|| ctx.native_pixels_per_point().unwrap_or(1.0));
        ctx.set_pixels_per_point(ppp);

        // セットアップダイアログ表示中はそれだけを描画。
        // setup_dialog は ui を受けるので、透明な CentralPanel で ui を用意する
        // （Window 自体は ui.ctx() に開くので見た目は変わらない）。
        if self.state.show_setup_dialog() {
            let actions = {
                let slices = self.state.slices();
                CentralPanel::default()
                    .frame(Frame::NONE)
                    .show(ctx, |ui| SetupDialogFeature::render(&slices, ui))
                    .inner
            };
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
        // 読み取りは Slices、Feature の状態は features（disjoint なフィールドを分けて借りる）
        let slices = self.state.slices();
        let features = &mut self.features;
        let mut actions = Vec::new();

        // 右側のプレイヤー情報パネル
        let side = SidePanel::right(PanelIds::PLAYER_INFO)
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    let head_text = RichText::new(slices.t(keys::SIDEBAR_PLAYERS))
                        .heading()
                        .color(Color32::WHITE);
                    ui.heading(head_text);
                });
                ui.separator();
                PlayerInfoFeature::render(&slices, ui)
            });
        actions.extend(side.inner);

        // 残りの中央領域
        let central = CentralPanel::default()
            .frame(Self::get_frame())
            .show(ctx, |ui| MainView::render(&slices, features, ui));
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
                AppAction::Wave(a) => Actions::new(&mut self.state).handle_wave(a),
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
        // UI 拡大率の操作（右から left_to_right で並ぶため、視覚順とは逆に積まれる）
        let mut scale_inc = false;
        let mut scale_dec = false;
        let mut scale_auto = false;

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

                        // UI 拡大率コントロール（現在の実効倍率を % 表示し ±/auto で操作）
                        ui.separator();
                        if ui
                            .small_button("auto")
                            .on_hover_text("DPI に追従")
                            .clicked()
                        {
                            scale_auto = true;
                        }
                        if ui.small_button("＋").clicked() {
                            scale_inc = true;
                        }
                        ui.label(format!("{:.0}%", ui.ctx().pixels_per_point() * 100.0));
                        if ui.small_button("−").clicked() {
                            scale_dec = true;
                        }
                        ui.label("🔍");
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
        // 現在の実効倍率を基準に増減（None=自動状態からでも自然に明示倍率へ移行する）
        let current_ppp = ctx.pixels_per_point();
        if scale_inc {
            self.state
                .set_ui_scale(current_ppp + AppConstants::UI_SCALE_STEP);
        }
        if scale_dec {
            self.state
                .set_ui_scale(current_ppp - AppConstants::UI_SCALE_STEP);
        }
        if scale_auto {
            self.state.reset_ui_scale();
        }
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::WINDOW_BG_COLOR_DARK)
            .stroke(Stroke::NONE)
    }
}
