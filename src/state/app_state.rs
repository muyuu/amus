use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use crate::assets::AssetManager;
use crate::models::player::PlayerId;
use crate::state::app_data::AppData;
use crate::state::location::DraggingLocation;
use crate::state::AppStorage;
use crate::state::SetupState;
use crate::state::StorageKeys;
use crate::{log_debug, models::*};

/// アプリケーション状態へのアクセスを提供する構造体
pub struct AppState {
    data: RefCell<AppData>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: RefCell::new(AppData::default()),
        }
    }
}

// 基本的なコンストラクタとデータアクセス
impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// アプリケーションデータへの可変参照を取得
    ///
    /// このメソッドは `state` 自体を借用しないため、
    /// `state.current_wave()` など他のメソッドと同時に使えます。
    fn data_mut(&self) -> RefMut<'_, AppData> {
        self.data.borrow_mut()
    }

    pub fn t(&self, key: &str) -> String {
        self.data.borrow().translator.t(key).to_string()
    }

    pub fn asset_manager(&self) -> Option<AssetManager> {
        let data = self.data.borrow();
        data.asset_manager.clone()
    }

    pub fn set_asset_manager(&self, asset_manager: AssetManager) {
        self.data.borrow_mut().asset_manager = Some(asset_manager);
    }
}

// ゲーム管理関連
impl AppState {
    pub fn start_new_game(&self) {
        self.data_mut().show_setup_dialog = true;
    }

    pub fn create_game_from_setup(&self) {
        let mut data = self.data_mut();
        let area = data.setup_state.selected_area.clone();
        let player_count = data.setup_state.player_count;
        let players = data
            .setup_state
            .players
            .iter()
            .take(player_count)
            .cloned()
            .collect();

        let mut game = Game::new(area, players);
        // とりあえず20ウェーブ作成
        for _ in 0..20 {
            game.add_wave();
        }
        data.game = Some(game);
        data.current_wave_index = 0;
        data.show_setup_dialog = false;
    }

    pub fn cancel_setup(&self) {
        self.data_mut().show_setup_dialog = false;
    }

    pub fn reset_game(&self) {
        // setup_state は変えなくて良いケースが多いはずなので保持
        let setup_state = self.data.borrow().setup_state.clone();
        *self.data.borrow_mut() = AppData::default();
        self.data.borrow_mut().setup_state = setup_state;
    }

    pub fn select_wave(&self, index: usize) {
        self.data_mut().current_wave_index = index;
    }

    pub fn game(&self) -> Option<Game> {
        let data = self.data.borrow();
        data.game.clone()
    }

    pub fn area(&self) -> Option<Area> {
        let game = self.game();
        if let Some(game) = game {
            Some(game.area)
        } else {
            None
        }
    }
}

// プレイヤー操作関連
impl AppState {
    pub fn toggle_player_state(&self, player_id: PlayerId) {
        if let Some(game) = self.data.borrow_mut().game.as_mut() {
            game.players.iter_mut().for_each(|p| {
                if p.id == player_id {
                    p.state = p.state.next();
                }
            });
        }
    }

    pub fn adjust_player_count(&self, new_count: usize) {
        let mut data = self.data_mut();
        let old_count = data.setup_state.players.len();
        data.setup_state.player_count = new_count;

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
                // 現在のプレイヤーが使ってない色を選択
                let used_colors = data
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

                data.setup_state.players.push(Player::new(
                    Role::Crew,
                    color,
                    format!("Player {}", i + 1),
                ));
            }
        }

        // 余計なプレイヤーを削除
        data.setup_state.players.truncate(new_count);
    }

    pub fn players(&self) -> Option<Vec<Player>> {
        let game = self.game();
        if let Some(game) = game {
            Some(game.players)
        } else {
            None
        }
    }

    pub fn player(&self, player_id: PlayerId) -> Option<Player> {
        let players = self.players()?;
        players.into_iter().find(|p| p.id == player_id)
    }

    fn player_mut(&self, player_id: PlayerId) -> Option<RefMut<'_, Player>> {
        let mut data = self.data_mut();
        let game = data.game.as_mut()?;
        let index = game.players.iter().position(|p| p.id == player_id)?;
        Some(std::cell::RefMut::map(data, |d| {
            &mut d.game.as_mut().unwrap().players[index]
        }))
    }

    pub fn player_name_editing(&self, player_id: PlayerId) -> bool {
        let data = self.data.borrow();
        data.editing_name_player_id == Some(player_id)
    }

    pub fn toggle_player_name_editing(&self, player_id: PlayerId) {
        let mut data = self.data_mut();
        if data.editing_name_player_id == Some(player_id) {
            data.editing_name_player_id = None;
        } else {
            data.editing_name_player_id = Some(player_id);
        }
    }

    pub fn update_player_name(&self, id: PlayerId, name: String) {
        self.data
            .borrow_mut()
            .setup_state
            .players
            .iter_mut()
            .for_each(|p| {
                if p.id == id {
                    p.name = name.clone();
                }
            });

        if let Some(mut game) = self.game() {
            let mut players = game.players;
            players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.name = name.clone();
                }
            });
            game.players = players;
            self.data.borrow_mut().game = Some(game)
        }
    }

    #[allow(dead_code)]
    /// 色の変更を試みる（重複している場合は変更を拒否）
    pub fn try_update_player_color(&self, id: PlayerId, color: Color) -> Result<(), String> {
        // setup_state での重複チェック
        let is_duplicate = self
            .data
            .borrow()
            .setup_state
            .players
            .iter()
            .any(|p| p.id != id && p.color == color);

        if is_duplicate {
            return Err(format!("色 {} は既に使用されています", color.name()));
        }

        // 重複がない場合は更新
        self.data
            .borrow_mut()
            .setup_state
            .players
            .iter_mut()
            .for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                }
            });

        // game が存在する場合も更新
        if let Some(mut game) = self.game() {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                }
            });
            self.data.borrow_mut().game = Some(game);
        }

        Ok(())
    }

    /// 色を強制的に変更する（重複している場合は他のプレイヤーと色をスワップ）
    pub fn force_update_player_color(&self, id: PlayerId, color: Color) {
        let (other_player_id, current_color) = {
            let data = self.data.borrow();

            // 対象の色を使っている他のプレイヤーを探す
            let other_id = data
                .setup_state
                .players
                .iter()
                .find(|p| p.id != id && p.color == color)
                .map(|p| p.id);

            // 対象プレイヤーの現在の色を取得
            let current = data
                .setup_state
                .players
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.color.clone());

            (other_id, current)
        };

        // setup_state を更新
        self.data
            .borrow_mut()
            .setup_state
            .players
            .iter_mut()
            .for_each(|p| {
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
        if let Some(mut game) = self.game() {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                } else if Some(p.id) == other_player_id {
                    if let Some(ref swap_color) = current_color {
                        p.color = swap_color.clone();
                    }
                }
            });
            self.data.borrow_mut().game = Some(game);
        }
    }

    pub fn update_player_button(&self, id: PlayerId, done: bool) {
        // これはリセット時に初期化したいので setup_state は更新しない
        if let Some(mut game) = self.game() {
            let mut players = game.players;
            players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.done_button = done;
                }
            });
            game.players = players;
            self.data.borrow_mut().game = Some(game);
        }
    }

    pub fn toggle_comms(&self, id: PlayerId) {
        let mut player = match self.player_mut(id) {
            Some(p) => p,
            None => return,
        };

        player.resolved_comms = !player.resolved_comms;
    }

    pub fn toggle_lights(&self, id: PlayerId) {
        let mut player = match self.player_mut(id) {
            Some(p) => p,
            None => return,
        };

        player.resolved_lights = !player.resolved_lights;
    }

    pub fn toggle_o2(&self, id: PlayerId) {
        let mut player = match self.player_mut(id) {
            Some(p) => p,
            None => return,
        };

        player.resolved_o2 = !player.resolved_o2;
    }

    pub fn toggle_reactor(&self, id: PlayerId) {
        let mut player = match self.player_mut(id) {
            Some(p) => p,
            None => return,
        };

        player.resolved_reactor = !player.resolved_reactor;
    }
}

// UI状態管理関連
impl AppState {
    pub fn current_wave_index(&self) -> usize {
        let i = self.data.borrow().current_wave_index;
        i
    }

    pub fn show_debug_view(&self) -> bool {
        self.data.borrow().show_debug_view
    }

    pub fn show_setup_dialog(&self) -> bool {
        self.data.borrow().show_setup_dialog
    }

    pub fn erase_mode(&self) -> bool {
        self.data.borrow().erase_mode
    }

    pub fn set_erase_mode(&self, mode: bool) {
        self.data.borrow_mut().erase_mode = mode;
    }

    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.borrow().selected_player_id
    }

    pub fn set_selected_player_id(&self, player_id: Option<PlayerId>) {
        self.data.borrow_mut().selected_player_id = player_id;
    }

    pub fn setup_state(&self) -> Ref<'_, SetupState> {
        Ref::map(self.data.borrow(), |data| &data.setup_state)
    }

    pub fn setup_state_mut(&self) -> RefMut<'_, SetupState> {
        RefMut::map(self.data.borrow_mut(), |data| &mut data.setup_state)
    }

    pub fn set_selected_area(&self, area: Area) {
        self.data.borrow_mut().setup_state.selected_area = area;
    }

    pub fn toggle_setup_dialog(&self) {
        let mut data = self.data_mut();
        data.show_setup_dialog = !data.show_setup_dialog;
    }

    pub fn toggle_debug_view(&self) {
        let mut data = self.data_mut();
        data.show_debug_view = !data.show_debug_view;
    }
}

// ドラッグ&ドロップ関連
impl AppState {
    pub fn dragging_player_id(&self) -> Option<PlayerId> {
        self.data.borrow().dragging_player_id
    }

    pub fn set_dragging_player_id(&self, player_id: Option<PlayerId>) {
        self.data.borrow_mut().dragging_player_id = player_id;
    }

    pub fn dragging_location(&self) -> Option<DraggingLocation> {
        self.data.borrow().dragging_location
    }

    pub fn set_dragging_location(&self, location: Option<DraggingLocation>) {
        self.data.borrow_mut().dragging_location = location;
    }

    pub fn spawn_locations(&self) -> Option<HashMap<PlayerId, Point>> {
        match self.current_wave() {
            Ok(wave) => Some(wave.spawn_locations.clone()),
            _ => None,
        }
    }

    pub fn add_spawn_location(&self, player_id: PlayerId, point: Point) {
        let mut wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        wave.spawn_locations.insert(player_id, point);
    }

    pub fn end_locations(&self) -> Option<HashMap<PlayerId, Point>> {
        match self.current_wave() {
            Ok(wave) => Some(wave.end_locations.clone()),
            _ => None,
        }
    }

    pub fn add_end_location(&self, player_id: PlayerId, point: Point) {
        let mut wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        wave.end_locations.insert(player_id, point);
    }
}

// ウェーブとルート管理関連
impl AppState {
    /// 現在の wave への不変参照を取得
    pub fn current_wave(&self) -> Result<Ref<'_, Wave>, String> {
        let data = self.data.borrow();
        let wave_index = data.current_wave_index;
        if data.game.is_none() {
            return Err("Game not found".to_string());
        }
        // Ref::map を使って Ref<AppData> から Ref<Wave> を作成
        Ok(Ref::map(data, |d| {
            d.game
                .as_ref()
                .and_then(|game| game.get_wave(wave_index))
                .expect("Wave should exist at this index")
        }))
    }

    /// 現在の wave への可変参照を取得
    fn current_wave_mut(&self) -> Result<RefMut<'_, Wave>, String> {
        let data = self.data.borrow_mut();
        let wave_index = data.current_wave_index;
        if data.game.is_none() {
            return Err("Game not found".to_string());
        }
        // RefMut::map を使って RefMut<AppData> から RefMut<Wave> を作成
        Ok(std::cell::RefMut::map(data, |d| {
            d.game
                .as_mut()
                .and_then(|game| game.get_wave_mut(wave_index))
                .expect("Wave should exist at this index")
        }))
    }

    pub fn routes(&self) -> Option<Vec<Route>> {
        match self.current_wave() {
            Ok(wave) => Some(wave.routes.clone()),
            _ => None,
        }
    }

    pub fn routes_mut(&self) -> Option<RefMut<'_, Vec<Route>>> {
        match self.current_wave_mut() {
            Ok(wave) => Some(std::cell::RefMut::map(wave, |w| &mut w.routes)),
            _ => None,
        }
    }

    pub fn last_route_mut(&self) -> Option<RefMut<'_, Route>> {
        match self.routes_mut() {
            Some(routes) => {
                let len = routes.len();
                if len == 0 {
                    return None;
                }
                Some(std::cell::RefMut::map(routes, |r| &mut r[len - 1]))
            }
            None => None,
        }
    }

    pub fn push_route(&self, route: Route) {
        let mut wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        wave.routes.push(route);
    }
}

// ストレージ関連
impl AppState {
    /// ストレージからアプリケーション状態を復元
    pub fn load_from_storage(&self, storage: Option<&dyn eframe::Storage>) -> Result<(), String> {
        // セットアップ状態を復元
        if let Ok(Some(setup_state)) =
            AppStorage::get::<SetupState>(storage, StorageKeys::SETUP_STATE)
        {
            log_debug!("AppState", "SetupState をストレージから復元");
            self.data.borrow_mut().setup_state = setup_state;
        }

        // ゲーム状態を復元
        if let Ok(Some(game)) = AppStorage::get::<Game>(storage, StorageKeys::GAME) {
            log_debug!("AppState", "Game をストレージから復元");
            self.data.borrow_mut().game = Some(game);
        }

        // 現在のウェーブインデックスを復元
        if let Ok(Some(wave_index)) =
            AppStorage::get::<usize>(storage, StorageKeys::CURRENT_WAVE_INDEX)
        {
            log_debug!("AppState", "現在のターンをストレージから復元");
            self.data.borrow_mut().current_wave_index = wave_index;
        }

        // UI状態を復元
        if let Ok(Some(show_debug)) = AppStorage::get::<bool>(storage, StorageKeys::SHOW_DEBUG_VIEW)
        {
            self.data.borrow_mut().show_debug_view = show_debug;
        }

        if let Ok(Some(erase_mode)) = AppStorage::get::<bool>(storage, StorageKeys::ERASE_MODE) {
            self.data.borrow_mut().erase_mode = erase_mode;
        }

        Ok(())
    }

    /// アプリケーション状態をストレージに保存
    pub fn save_to_storage(&self, storage: Option<&mut dyn eframe::Storage>) -> Result<(), String> {
        use crate::state::storage::AppStorage;
        use crate::state::storage_keys::StorageKeys;

        let data = self.data.borrow();

        AppStorage::save_multiple(storage, |s| {
            // セットアップ状態を保存
            AppStorage::set(Some(s), StorageKeys::SETUP_STATE, &data.setup_state)?;

            // ゲーム状態を保存
            if let Some(ref game) = data.game {
                AppStorage::set(Some(s), StorageKeys::GAME, game)?;
            }

            // 現在のウェーブインデックスを保存
            AppStorage::set(
                Some(s),
                StorageKeys::CURRENT_WAVE_INDEX,
                &data.current_wave_index,
            )?;

            // UI状態を保存
            AppStorage::set(Some(s), StorageKeys::SHOW_DEBUG_VIEW, &data.show_debug_view)?;
            AppStorage::set(Some(s), StorageKeys::ERASE_MODE, &data.erase_mode)?;

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
    pub fn clear_game_data(&self, storage: Option<&mut dyn eframe::Storage>) -> Result<(), String> {
        use crate::state::storage::AppStorage;
        use crate::state::storage_keys::StorageKeys;

        AppStorage::save_multiple(storage, |s| {
            // ゲーム関連のデータを削除
            s.set_string(StorageKeys::GAME, "".to_string());
            s.set_string(StorageKeys::CURRENT_WAVE_INDEX, "0".to_string());
            Ok(())
        })?;

        // メモリ上のデータもクリア
        let mut data = self.data.borrow_mut();
        data.game = None;
        data.current_wave_index = 0;

        Ok(())
    }
}
