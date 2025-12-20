use std::collections::HashMap;

use crate::log_debug;
use crate::models::*;
use crate::state::app_data::AppData;
use crate::state::slices::Slices;
use crate::state::AppStorage;
use crate::state::StorageKeys;

/// アプリケーション状態へのアクセスを提供する構造体
pub struct AppState {
    data: AppData,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: AppData::default(),
        }
    }
}

// 基本的なコンストラクタとデータアクセス
impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 読み取り専用のSlicesを取得
    ///
    /// ViewはこのSlicesを通じてデータにアクセスする。
    /// 書き込みはActionsを通じて行う。
    pub fn slices(&self) -> Slices<'_> {
        Slices::new(&self.data)
    }

    pub fn t(&self, key: &str) -> String {
        self.data.translator.t(key).to_string()
    }
}

// ゲーム管理関連
impl AppState {
    pub fn start_new_game(&mut self) {
        self.data.show_setup_dialog = true;
    }

    pub fn create_game_from_setup(&mut self) {
        let area = self.data.setup_state.selected_area.clone();
        let player_count = self.data.setup_state.player_count;
        let players = self
            .data
            .setup_state
            .players
            .iter()
            .take(player_count)
            .cloned()
            .collect();

        let mut new_game = Game::new(area, players);
        // とりあえず20ウェーブ作成
        for _ in 0..20 {
            new_game.waves.push(Wave::default());
        }
        self.data.game = Some(new_game);
        self.data.current_wave_index = 0;
        self.data.show_setup_dialog = false;
    }

    pub fn cancel_setup(&mut self) {
        self.data.show_setup_dialog = false;
    }

    pub fn reset_game(&mut self) {
        // setup_state は変えなくて良いケースが多いはずなので保持
        let setup_state = self.data.setup_state.clone();
        self.data = AppData::default();
        self.data.setup_state = setup_state;
    }

    pub fn select_wave(&mut self, index: usize) {
        self.data.current_wave_index = index;
    }

    pub fn game(&self) -> Option<&Game> {
        self.data.game.as_ref()
    }

    pub fn area(&self) -> Option<Area> {
        self.game().map(|game| game.area.clone())
    }
}

// プレイヤー操作関連
impl AppState {
    pub fn toggle_player_state(&mut self, player_id: PlayerId) {
        if let Some(game) = self.data.game.as_mut() {
            game.players.iter_mut().for_each(|p| {
                if p.id == player_id {
                    let next = match p.state {
                        PlayerState::Alive => PlayerState::Killed,
                        PlayerState::Killed => PlayerState::Ejected,
                        PlayerState::Ejected => PlayerState::Alive,
                    };
                    p.state = next;
                }
            });
        }
    }

    pub fn adjust_player_count(&mut self, new_count: usize) {
        let old_count = self.data.setup_state.players.len();
        self.data.setup_state.player_count = new_count;

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

            for _i in old_count..new_count {
                // 現在のプレイヤーが使ってない色を選択
                let used_colors = self
                    .data
                    .setup_state
                    .players
                    .iter()
                    .map(|p| p.color.clone())
                    .collect::<Vec<Color>>();
                let color = available_colors
                    .iter()
                    .find(|c| !used_colors.contains(c))
                    .cloned()
                    .unwrap_or(Color::Red);

                self.data
                    .setup_state
                    .players
                    .push(Player::new(Role::Crew, color, "".to_string()));
            }
        }
    }

    pub fn players(&self) -> Option<Vec<Player>> {
        self.game().map(|game| game.players.clone())
    }

    pub fn player(&self, player_id: PlayerId) -> Option<Player> {
        let players = self.players()?;
        players.into_iter().find(|p| p.id == player_id)
    }

    fn player_mut(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        let game = self.data.game.as_mut()?;
        game.players.iter_mut().find(|p| p.id == player_id)
    }

    pub fn player_name_editing(&self, player_id: PlayerId) -> bool {
        self.data.editing_name_player_id == Some(player_id)
    }

    pub fn toggle_player_name_editing(&mut self, player_id: PlayerId) {
        if self.data.editing_name_player_id == Some(player_id) {
            self.data.editing_name_player_id = None;
        } else {
            self.data.editing_name_player_id = Some(player_id);
        }
    }

    pub fn update_player_name(&mut self, id: PlayerId, name: String) {
        // setup_stateを更新
        self.data.setup_state.players.iter_mut().for_each(|p| {
            if p.id == id {
                p.name = name.clone();
            }
        });

        // gameを更新
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.name = name.clone();
                }
            });
        }
    }

    #[allow(dead_code)]
    /// 色の変更を試みる（重複している場合は変更を拒否）
    pub fn try_update_player_color(&mut self, id: PlayerId, color: Color) -> Result<(), String> {
        // setup_state での重複チェック
        let is_duplicate = self
            .players()
            .unwrap_or_default()
            .iter()
            .any(|p| p.id != id && p.color == color);

        if is_duplicate {
            return Err(format!("色 {} は既に使用されています", color.name()));
        }

        // 重複がない場合は更新
        self.data.setup_state.players.iter_mut().for_each(|p| {
            if p.id == id {
                p.color = color.clone();
            }
        });

        // game が存在する場合も更新
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                }
            });
        }

        Ok(())
    }

    /// 色を強制的に変更する（重複している場合は他のプレイヤーと色をスワップ）
    pub fn force_update_player_color(&mut self, id: PlayerId, color: Color) {
        // 対象の色を使っている他のプレイヤーを探す
        let other_player_id = self
            .data
            .setup_state
            .players
            .iter()
            .find(|p| p.id != id && p.color == color)
            .map(|p| p.id);

        // 対象プレイヤーの現在の色を取得
        let current_color = self
            .data
            .setup_state
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.color.clone());

        // setup_state と game を更新
        self.data.setup_state.players.iter_mut().for_each(|p| {
            if p.id == id {
                // 対象プレイヤーの色を変更
                p.color = color.clone();
            } else if Some(p.id) == other_player_id {
                // 重複していたプレイヤーの色を対象プレイヤーの元の色に変更
                if let Some(ref swap_color) = current_color {
                    p.color = swap_color.clone();
                }
            }
        });

        // game が存在する場合も更新
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                } else if Some(p.id) == other_player_id {
                    if let Some(ref swap_color) = current_color {
                        p.color = swap_color.clone();
                    }
                }
            });
        }
    }

    pub fn update_player_button(&mut self, id: PlayerId, done: bool) {
        // これはリセット時に初期化したいので setup_state は更新しない
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.done_button = done;
                }
            });
        }
    }

    pub fn toggle_comms(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_comms = !player.resolved_comms;
        }
    }

    pub fn toggle_lights(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_lights = !player.resolved_lights;
        }
    }

    pub fn toggle_o2(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_o2 = !player.resolved_o2;
        }
    }

    pub fn toggle_reactor(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_reactor = !player.resolved_reactor;
        }
    }
}

// UI状態管理関連
impl AppState {
    pub fn current_wave_index(&self) -> usize {
        self.data.current_wave_index
    }

    pub fn show_debug_view(&self) -> bool {
        self.data.show_debug_view
    }

    pub fn show_setup_dialog(&self) -> bool {
        self.data.show_setup_dialog
    }

    pub fn erase_mode(&self) -> bool {
        self.data.erase_mode
    }

    pub fn set_erase_mode(&mut self, mode: bool) {
        self.data.erase_mode = mode;
    }

    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.selected_player_id
    }

    pub fn set_selected_player_id(&mut self, player_id: Option<PlayerId>) {
        self.data.selected_player_id = player_id;
    }

    pub fn setup_state(&self) -> &SetupState {
        &self.data.setup_state
    }

    pub fn set_selected_area(&mut self, area: Area) {
        self.data.setup_state.selected_area = area;
    }

    pub fn toggle_setup_dialog(&mut self) {
        self.data.show_setup_dialog = !self.data.show_setup_dialog;
    }

    #[cfg(debug_assertions)]
    pub fn toggle_debug_view(&mut self) {
        self.data.show_debug_view = !self.data.show_debug_view;
    }
}

// ドラッグ&ドロップ関連
impl AppState {
    pub fn dragging_player_id(&self) -> Option<PlayerId> {
        self.data.dragging_player_id
    }

    pub fn set_dragging_player_id(&mut self, player_id: Option<PlayerId>) {
        self.data.dragging_player_id = player_id;
    }

    pub fn dragging_location(&self) -> Option<DraggingLocation> {
        self.data.dragging_location
    }

    pub fn set_dragging_location(&mut self, location: Option<DraggingLocation>) {
        self.data.dragging_location = location;
    }

    pub fn locations(&self, location_type: LocationType) -> Option<HashMap<PlayerId, Point>> {
        self.current_wave().ok().map(|w| match location_type {
            LocationType::Spawn => w.spawn_locations.clone(),
            LocationType::End => w.end_locations.clone(),
        })
    }

    pub fn add_location(&mut self, location_type: LocationType, player_id: PlayerId, point: Point) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        match location_type {
            LocationType::Spawn => wave.spawn_locations.insert(player_id, point),
            LocationType::End => wave.end_locations.insert(player_id, point),
        };
    }

    pub fn remove_location(&mut self, location_type: LocationType, player_id: PlayerId) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        match location_type {
            LocationType::Spawn => wave.spawn_locations.remove(&player_id),
            LocationType::End => wave.end_locations.remove(&player_id),
        };
    }
}

// ウェーブとルート管理関連
impl AppState {
    /// 現在の wave への不変参照を取得
    pub fn current_wave(&self) -> Result<&Wave, String> {
        let wave_index = self.data.current_wave_index;
        let game = self.data.game.as_ref().ok_or("Game not found")?;
        game.get_wave(wave_index)
            .ok_or_else(|| "Wave not found".to_string())
    }

    /// 現在の wave への可変参照を取得
    fn current_wave_mut(&mut self) -> Result<&mut Wave, String> {
        let wave_index = self.data.current_wave_index;
        let game = self.data.game.as_mut().ok_or("Game not found")?;
        game.get_wave_mut(wave_index)
            .ok_or_else(|| "Wave not found".to_string())
    }

    pub fn routes(&self) -> Option<Vec<Route>> {
        self.current_wave().ok().map(|wave| wave.routes.clone())
    }

    pub fn push_route(&mut self, route: Route) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        wave.routes.push(route);
    }

    pub fn add_point_to_last_route(&mut self, point: Point) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        let last_route = match wave.routes.last_mut() {
            Some(r) => r,
            None => return,
        };

        let lines = last_route.lines_mut();
        let last_line = match lines.last_mut() {
            Some(line) => line,
            None => return,
        };

        // 直前のポイントと同じ場合は追加しない
        let last_point = last_line.last();
        if let Some(lp) = last_point {
            if lp.x == point.x && lp.y == point.y {
                return;
            }
        }
        last_line.push(point);
    }
}

// ストレージ関連
impl AppState {
    /// ストレージからアプリケーション状態を復元
    pub fn load_from_storage(&mut self, storage: Option<&dyn eframe::Storage>) -> Result<(), String> {
        // セットアップ状態を復元
        if let Ok(Some(setup_state)) =
            AppStorage::get::<SetupState>(storage, StorageKeys::SETUP_STATE)
        {
            log_debug!("AppState", "SetupState をストレージから復元");
            self.data.setup_state = setup_state;
        }

        // ゲーム状態を復元
        if let Ok(Some(game)) = AppStorage::get::<Game>(storage, StorageKeys::GAME) {
            log_debug!("AppState", "Game をストレージから復元");
            self.data.game = Some(game);
        }

        // 現在のウェーブインデックスを復元
        if let Ok(Some(wave_index)) =
            AppStorage::get::<usize>(storage, StorageKeys::CURRENT_WAVE_INDEX)
        {
            log_debug!("AppState", "現在のターンをストレージから復元");
            self.data.current_wave_index = wave_index;
        }

        // UI状態を復元
        if let Ok(Some(show_debug)) = AppStorage::get::<bool>(storage, StorageKeys::SHOW_DEBUG_VIEW)
        {
            self.data.show_debug_view = show_debug;
        }

        if let Ok(Some(erase_mode)) = AppStorage::get::<bool>(storage, StorageKeys::ERASE_MODE) {
            self.data.erase_mode = erase_mode;
        }

        Ok(())
    }

    /// アプリケーション状態をストレージに保存
    pub fn save_to_storage(&self, storage: Option<&mut dyn eframe::Storage>) -> Result<(), String> {
        use crate::state::storage::AppStorage;
        use crate::state::storage_keys::StorageKeys;

        AppStorage::save_multiple(storage, |s| {
            // セットアップ状態を保存
            AppStorage::set(Some(s), StorageKeys::SETUP_STATE, &self.data.setup_state)?;

            // ゲーム状態を保存
            if let Some(ref game) = self.data.game {
                AppStorage::set(Some(s), StorageKeys::GAME, game)?;
            }

            // 現在のウェーブインデックスを保存
            AppStorage::set(
                Some(s),
                StorageKeys::CURRENT_WAVE_INDEX,
                &self.data.current_wave_index,
            )?;

            // UI状態を保存
            AppStorage::set(Some(s), StorageKeys::SHOW_DEBUG_VIEW, &self.data.show_debug_view)?;
            AppStorage::set(Some(s), StorageKeys::ERASE_MODE, &self.data.erase_mode)?;

            Ok(())
        })
    }

    #[allow(dead_code)]
    /// ストレージから特定のキーのデータを読み込み
    pub fn load_value<T>(
        &self,
        storage: Option<&dyn eframe::Storage>,
        key: &str,
    ) -> Result<Option<T>, String>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        use crate::state::storage::AppStorage;
        AppStorage::get(storage, key)
    }

    #[allow(dead_code)]
    /// ストレージに特定のキーでデータを保存
    pub fn save_value<T>(
        &self,
        storage: Option<&mut dyn eframe::Storage>,
        key: &str,
        value: &T,
    ) -> Result<(), String>
    where
        T: serde::Serialize,
    {
        use crate::state::storage::AppStorage;
        AppStorage::set(storage, key, value)
    }

    #[allow(dead_code)]
    /// ゲームデータのみをクリア（設定は保持）
    pub fn clear_game_data(
        &mut self,
        storage: Option<&mut dyn eframe::Storage>,
    ) -> Result<(), String> {
        use crate::state::storage::AppStorage;
        use crate::state::storage_keys::StorageKeys;

        AppStorage::save_multiple(storage, |s| {
            // ゲーム関連のデータを削除
            s.set_string(StorageKeys::GAME, "".to_string());
            s.set_string(StorageKeys::CURRENT_WAVE_INDEX, "0".to_string());
            Ok(())
        })?;

        // メモリ上のデータもクリア
        self.data.game = None;
        self.data.current_wave_index = 0;

        Ok(())
    }
}
