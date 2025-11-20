use crate::assets::AssetManager;
use crate::i18n::{Language, Translator};
use crate::models::*;

// デフォルトのプレイヤー人数定数
const DEFAULT_PLAYER_COUNT: usize = 8;

#[derive(Debug, Clone)]
pub struct PlayerSetup {
    pub name: String,
    pub color: Color,
    #[allow(dead_code)]
    pub role: Option<Role>, // セットアップ時は未決定
}

impl PlayerSetup {
    pub fn new(index: usize, color: Color) -> Self {
        Self {
            name: format!("Player{}", index + 1),
            color,
            role: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetupState {
    pub selected_area_id: String,
    pub selected_area_name: String,
    pub player_count: usize,
    pub players: Vec<PlayerSetup>,
}

impl Default for SetupState {
    fn default() -> Self {
        let default_colors = [
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Pink,
            Color::Orange,
            Color::Yellow,
            Color::Black,
            Color::White,
            Color::Purple,
            Color::Brown,
            Color::Cyan,
            Color::Lime,
            Color::Maroon,
            Color::Rose,
            Color::Banana,
        ];

        let mut players = Vec::new();
        for i in 0..DEFAULT_PLAYER_COUNT {
            // デフォルトの人数分
            let color = default_colors
                .get(i % default_colors.len())
                .cloned()
                .unwrap_or(Color::Red);
            players.push(PlayerSetup::new(i, color));
        }

        Self {
            selected_area_id: "airship".to_string(),
            selected_area_name: "Airship".to_string(),
            player_count: DEFAULT_PLAYER_COUNT,
            players,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub game: Option<Game>,
    pub current_wave_index: usize,
    pub selected_user_id: Option<usize>,
    pub temp_points: Vec<Point>, // 描画中の一時的なポイント
    pub show_setup_dialog: bool,
    pub setup_state: SetupState, // ゲーム設定の状態
    pub translator: Translator,
    pub asset_manager: Option<AssetManager>, // 画像リソース管理
    #[allow(dead_code)]
    pub show_turn_menu: bool, // ターンメニューの表示状態
    pub dragging_user_id: Option<usize>,     // ドラッグ中のユーザーID
    pub show_debug_view: bool,               // デバッグビューの表示状態
    pub dragging_location: Option<DraggingLocation>, // ドラッグ中の位置（出現位置 or 終了時位置）
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    Spawn, // 出現位置
    End,   // 終了時位置
}

#[derive(Debug, Clone, Copy)]
pub struct DraggingLocation {
    pub location_type: LocationType,
    pub user_id: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            game: None,
            current_wave_index: 0,
            selected_user_id: None,
            temp_points: Vec::new(),
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

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_assets(&mut self, ctx: &egui::Context) {
        if self.asset_manager.is_none() {
            self.asset_manager = Some(AssetManager::new(ctx));
        }
    }

    pub fn set_language(&mut self, language: Language) {
        self.translator.set_language(language);
    }

    pub fn current_language(&self) -> Language {
        self.translator.current_language()
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.translator.t(key)
    }

    pub fn start_new_game(&mut self) {
        self.show_setup_dialog = true;
    }

    pub fn create_game_from_setup(&mut self) {
        let area = Area::new(
            self.setup_state.selected_area_name.clone(),
            self.setup_state.selected_area_id.clone(),
        );

        // セットアップ済みプレイヤーからゲーム用ユーザーを作成
        let mut users = Vec::new();
        let player_count = self.setup_state.player_count;
        let imposter_count = (player_count as f32 * 0.2).ceil() as usize; // 約20%をインポスター

        for (i, player_setup) in self
            .setup_state
            .players
            .iter()
            .enumerate()
            .take(player_count)
        {
            // 最初の数人をインポスター、残りをクルーとする
            let role = if i < imposter_count {
                Role::Imposter
            } else {
                Role::Crew
            };
            users.push(User::new(
                role,
                player_setup.color.clone(),
                player_setup.name.clone(),
            ));
        }

        let mut game = Game::new(area, users);
        game.add_wave(); // 最初のターンを作成
        game.add_wave(); // 2番目のターンを作成
        self.game = Some(game);
        self.current_wave_index = 0;
        self.show_setup_dialog = false;
    }

    pub fn cancel_setup(&mut self) {
        self.show_setup_dialog = false;
    }

    pub fn adjust_player_count(&mut self, new_count: usize) {
        let old_count = self.setup_state.players.len();
        self.setup_state.player_count = new_count;

        if new_count > old_count {
            // プレイヤーを追加
            let available_colors = [
                Color::Red,
                Color::Blue,
                Color::Green,
                Color::Pink,
                Color::Orange,
                Color::Yellow,
                Color::Black,
                Color::White,
                Color::Purple,
                Color::Brown,
                Color::Cyan,
                Color::Lime,
                Color::Maroon,
                Color::Rose,
                Color::Banana,
                Color::Gray,
                Color::Tan,
                Color::Coral,
            ];

            for i in old_count..new_count {
                let color = available_colors.get(i).cloned().unwrap_or(Color::Red);
                self.setup_state.players.push(PlayerSetup::new(i, color));
            }
        } else if new_count < old_count {
            // プレイヤーを削除
            self.setup_state.players.truncate(new_count);
        }
    }

    pub fn select_wave(&mut self, index: usize) {
        self.current_wave_index = index;
        self.temp_points.clear();
    }

    pub fn add_new_wave(&mut self) {
        if let Some(game) = &mut self.game {
            game.add_wave();
            self.current_wave_index = game.waves.len() - 1;
        }
    }

    pub fn select_user(&mut self, user_id: usize) {
        self.selected_user_id = Some(user_id);
    }
}
