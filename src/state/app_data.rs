use crate::assets::AssetManager;
use crate::i18n::{Language, Translator};
use crate::models::*;
use crate::state::location::DraggingLocation;
use crate::state::setup_state::SetupState;

/// 実際のアプリケーション状態を保持する構造体
pub struct AppData {
    // 画像リソース管理
    pub asset_manager: Option<AssetManager>,

    pub current_wave_index: usize,

    // ドラッグ中の位置（出現位置 or 終了時位置）
    pub dragging_location: Option<DraggingLocation>,

    // ドラッグ中のユーザーID
    pub dragging_user_id: Option<usize>,

    // 消しゴムモードの状態
    pub erase_mode: bool,

    pub game: Option<Game>,

    pub selected_user_id: Option<usize>,

    // ゲーム設定の状態
    pub setup_state: SetupState,

    // デバッグビューの表示状態
    pub show_debug_view: bool,

    pub show_setup_dialog: bool,

    #[allow(dead_code)]
    pub show_turn_menu: bool, // ターンメニューの表示状態

    pub translator: Translator,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            asset_manager: None, // 後で初期化
            current_wave_index: 0,
            dragging_location: None, // ドラッグ中の位置はNone
            dragging_user_id: None,  // ドラッグ中はNone
            erase_mode: false,       // 初期状態では消しゴムモードオフ
            game: None,
            selected_user_id: None,
            setup_state: SetupState::default(),
            show_setup_dialog: true, // 起動時はセットアップダイアログを表示
            show_turn_menu: false,   // 初期状態では非表示
            show_debug_view: false,  // 初期状態では非表示
            translator: Translator::new(Language::Japanese), // デフォルトは日本語
        }
    }
}
