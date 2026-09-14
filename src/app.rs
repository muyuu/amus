use egui::*;

use crate::app_action::AppAction;
use crate::constants::{AppConstants, PanelIds};
use crate::features::main::MainView;
use crate::features::player_info::PlayerInfoFeature;
use crate::features::settings::SettingsFeature;
use crate::features::setup_dialog::SetupDialogFeature;
use crate::features::StatefulFeatures;
use crate::i18n::keys;
use crate::log_error;
use crate::resources::Resources;
use crate::state::{Actions, AppState, SettingsAction};

pub struct AmusApp {
    state: AppState,
    // resources を読むのは native 専用の voice_memo（① update / ③ handle）のみ。
    // wasm、または voice_memo feature を切った native では未使用になる。
    #[cfg_attr(
        not(all(not(target_arch = "wasm32"), feature = "voice_memo")),
        allow(dead_code)
    )]
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

        // 起動時点の Resources は既定デバイスで組んであるため、保存されていた
        // 入力デバイス選択があれば読み込み後にここで反映する。
        #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
        if let Some(device) = app.state.slices().ui().input_device_name() {
            app.resources.reload_voice_recorder(Some(&device));
        }

        app
    }
}

impl eframe::App for AmusApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Err(e) = self.state.save_to_storage(Some(storage)) {
            log_error!("AppState", format!("ストレージへの保存に失敗: {}", e));
        }
    }

    /// 終了時（save の後）に録音・ダウンロード・書き起こしスレッドを確実に止める。
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
        self.features.voice_memo.shutdown(&mut self.resources);
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
        #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
        self.features.voice_memo.update(&mut self.resources, ctx);

        // ② 描画して Action を収集
        let mut actions = self.build_main_ui(ctx, frame);

        // 設定モーダルはゲーム画面を覆い隠さず、上に重ねて描く
        // （軌跡の太さなど、背後を見ながら変更したい設定があるため）。
        if self.state.show_settings() {
            let slices = self.state.slices();
            actions.extend(SettingsFeature::render(&slices, ctx));
        }

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

        // 残りの中央領域（設定・リセットのアイコンはここに重ねる。倍率やデバイスは
        // ゲームのライフサイクルと無関係なので、アプリシェルとして直接 state を触る）
        let mut toggle_settings = false;
        let mut toggle_setup = false;
        let central = CentralPanel::default()
            .frame(Self::get_frame())
            .show(ctx, |ui| {
                Self::render_shell_icons(&slices, ui, &mut toggle_settings, &mut toggle_setup);
                MainView::render(&slices, features, ui)
            });
        actions.extend(central.inner);

        if toggle_settings {
            self.state.toggle_settings();
        }
        if toggle_setup {
            self.state.toggle_setup_dialog();
        }

        actions
    }

    /// メイン画面右上の設定・リセットアイコン（アプリシェル）。
    fn render_shell_icons(
        slices: &crate::state::Slices<'_>,
        ui: &mut Ui,
        toggle_settings: &mut bool,
        toggle_setup: &mut bool,
    ) {
        // 通常の button() の倍サイズ（アイコン文字・当たり判定とも）。
        let icon_text_size = 28.0;
        let icon_button_size = Vec2::new(48.0, 48.0);
        // 画面端との余白。
        let margin = 8.0;

        ui.add_space(margin);
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(margin);
                let gear =
                    Button::new(RichText::new("⚙").size(icon_text_size)).min_size(icon_button_size);
                if ui
                    .add(gear)
                    .on_hover_text(slices.t(keys::SETTINGS_TITLE))
                    .clicked()
                {
                    *toggle_settings = true;
                }
                let reset = Button::new(RichText::new("🔄").size(icon_text_size))
                    .min_size(icon_button_size);
                if ui
                    .add(reset)
                    .on_hover_text(slices.t(keys::SETTINGS_RESET_GAME))
                    .clicked()
                {
                    *toggle_setup = true;
                }
            });
        });
        ui.add_space(margin);
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
                AppAction::Settings(a) => self.handle_settings_action(a),
                AppAction::Wave(a) => Actions::new(&mut self.state).handle_wave(a),
                #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
                AppAction::VoiceMemo(a) => self
                    .features
                    .voice_memo
                    .handle_action(&mut self.resources, a),
            }
        }
    }

    /// 設定 Action を dispatch する。
    ///
    /// 入力デバイスの切り替えだけはハードウェア（`Resources`）に触るため、
    /// `AppState` しか触れない汎用の `Actions` では完結しない。録音中なら
    /// 安全に止めてから録音リソースを作り直し、その上で選択内容を state へ保存する
    /// （ゲーム進行中なら次フレームで新デバイスの録音が自動的に再開する）。
    fn handle_settings_action(&mut self, action: SettingsAction) {
        #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
        if let SettingsAction::SetInputDevice(ref device) = action {
            self.features
                .voice_memo
                .stop_recording_for_hardware_change(&mut self.resources);
            self.resources.reload_voice_recorder(device.as_deref());
        }

        Actions::new(&mut self.state).handle_settings(action);
    }

    fn get_frame() -> Frame {
        Frame::NONE
            .fill(AppConstants::BG_COLOR)
            .stroke(Stroke::NONE)
    }
}
