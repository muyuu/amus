use crate::assets::AssetManager;
use crate::i18n::{Language, Translator};
use crate::models::*;
use crate::state::location::DraggingLocation;
use crate::state::setup_state::SetupState;

/// 実際のアプリケーション状態を保持する構造体
pub struct AppData {
    pub game: Option<Game>,

    // 画像リソース管理
    pub asset_manager: Option<AssetManager>,

    pub current_wave_index: usize,

    // ドラッグ中のユーザーID
    pub dragging_user_id: Option<usize>,

    // ドラッグ中の位置（出現位置 or 終了時位置）
    pub dragging_location: Option<DraggingLocation>,

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
            game: None,
            current_wave_index: 0,
            selected_user_id: None,
            show_setup_dialog: true, // 起動時はセットアップダイアログを表示
            setup_state: SetupState::default(),
            translator: Translator::new(Language::Japanese), // デフォルトは日本語
            asset_manager: None,                             // 後で初期化
            show_turn_menu: false,                           // 初期状態では非表示
            dragging_user_id: None,                          // ドラッグ中はNone
            show_debug_view: false,                          // 初期状態では非表示
            dragging_location: None,                         // ドラッグ中の位置はNone
        }
    }
}
