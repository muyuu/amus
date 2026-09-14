use crate::constants::AppConstants;
use crate::i18n::{Language, Translator};
use crate::models::player::PlayerId;
use crate::models::*;
use crate::state::SettingsTab;

/// 実際のアプリケーション状態を保持する構造体
///
/// フィールドは `pub(in crate::state)` に閉じてある。外部（Feature / View 等）からは
/// Slices で読み取り、Actions / AppState メソッドで書き込む規約をコンパイラに強制するため。
pub struct AppData {
    pub(in crate::state) current_wave_index: usize,

    // ドラッグ中の位置（出現位置 or 終了時位置）
    pub(in crate::state) dragging_location: Option<DraggingLocation>,

    // プレイヤーリストの player をドラッグしている場合のプレイヤーID
    pub(in crate::state) dragging_player_id: Option<PlayerId>,

    /// プレイヤー名編集中のプレイヤーID
    pub(in crate::state) editing_name_player_id: Option<PlayerId>,

    // 消しゴムモードの状態
    pub(in crate::state) erase_mode: bool,

    pub(in crate::state) game: Option<Game>,

    /// ゲームが作り直されたことを検知するための世代番号。
    /// 作り直しでプレイヤーもマップも同じことがあるため、内容では区別できない。
    /// セッション内でのみ意味を持ち、永続化しない。
    pub(in crate::state) game_generation: u64,

    pub(in crate::state) selected_player_id: Option<PlayerId>,

    /// 軌跡描画の線の太さ（ユーザー指定可能な範囲へクランプ済み）。
    pub(in crate::state) route_line_width: f32,

    // ゲーム設定の状態
    pub(in crate::state) setup_state: SetupState,

    // デバッグビューの表示状態
    pub(in crate::state) show_debug_view: bool,

    pub(in crate::state) show_setup_dialog: bool,

    // 設定モーダルの表示状態
    pub(in crate::state) show_settings: bool,

    // 設定モーダルで選択中のタブ
    pub(in crate::state) settings_tab: SettingsTab,

    /// UI 拡大率（pixels_per_point）。`None` はネイティブ DPI に追従する自動。
    /// `Some(x)` はユーザーが明示指定した倍率（設定時に有効範囲へクランプ済み）。
    pub(in crate::state) ui_scale: Option<f32>,

    pub(in crate::state) translator: Translator,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            current_wave_index: 0,
            dragging_location: None,      // ドラッグ中の位置はNone
            dragging_player_id: None,     // ドラッグ中はNone
            editing_name_player_id: None, // ユーザー名編集中はNone
            erase_mode: false,            // 初期状態では消しゴムモードオフ
            game: None,
            game_generation: 0,
            selected_player_id: None,
            route_line_width: AppConstants::ROUTE_LINE_WIDTH_DEFAULT,
            setup_state: SetupState::default(),
            show_setup_dialog: false,
            show_settings: false,
            settings_tab: SettingsTab::default(),
            show_debug_view: false,
            ui_scale: None, // 既定はネイティブ DPI に追従
            translator: Translator::new(Language::Japanese), // デフォルトは日本語
        }
    }
}
