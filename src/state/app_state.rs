use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use crate::assets::AssetManager;
use crate::models::player::PlayerId;
use crate::models::*;
use crate::state::app_data::AppData;
use crate::state::location::DraggingLocation;
use crate::state::setup_state::SetupState;

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

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// アプリケーションデータへの可変参照を取得
    ///
    /// このメソッドは `state` 自体を借用しないため、
    /// `state.current_wave()` など他のメソッドと同時に使えます。
    pub fn data_mut(&self) -> RefMut<'_, AppData> {
        self.data.borrow_mut()
    }

    pub fn toggle_player_state(&self, player_id: PlayerId) {
        self.data.borrow_mut().game.as_mut().map(|game| {
            game.players.iter_mut().for_each(|p| {
                if p.id == player_id {
                    p.state = p.state.next();
                }
            });
        });
    }

    pub fn t(&self, key: &str) -> String {
        self.data.borrow().translator.t(key).to_string()
    }

    pub fn start_new_game(&self) {
        self.data_mut().show_setup_dialog = true;
    }

    pub fn create_game_from_setup(&self) {
        let mut data = self.data_mut();
        let area = data.setup_state.selected_area.clone();
        let players = data.setup_state.players.clone();

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
                let color = available_colors.get(i).cloned().unwrap_or(Color::Red);
                data.setup_state.players.push(Player::new(
                    Role::Crew,
                    color,
                    format!("Player {}", i + 1),
                ));
            }
        } else if new_count < old_count {
            // プレイヤーを削除
            data.setup_state.players.truncate(new_count);
        }
    }

    pub fn select_wave(&self, index: usize) {
        self.data_mut().current_wave_index = index;
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
}

// 既存のコードとの互換性のため、フィールドへの直接アクセスを提供
impl AppState {
    pub fn reset_game(&self) {
        // setup_state は変えなくて良いケースが多いはずなので保持
        let setup_state = self.data.borrow().setup_state.clone();
        *self.data.borrow_mut() = AppData::default();
        self.data.borrow_mut().setup_state = setup_state;
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

    /// 現在の wave への不変参照を取得
    #[allow(dead_code)]
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
    pub fn current_wave_mut(&self) -> Result<RefMut<'_, Wave>, String> {
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

    pub fn asset_manager(&self) -> Option<AssetManager> {
        let data = self.data.borrow();
        data.asset_manager.clone()
    }

    pub fn set_asset_manager(&self, asset_manager: AssetManager) {
        self.data.borrow_mut().asset_manager = Some(asset_manager);
    }

    pub fn current_wave_index(&self) -> usize {
        let i = self.data.borrow().current_wave_index;
        i
    }

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

    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.borrow().selected_player_id
    }

    pub fn set_selected_player_id(&self, player_id: Option<PlayerId>) {
        self.data.borrow_mut().selected_player_id = player_id;
    }

    pub fn setup_state(&self) -> Ref<'_, SetupState> {
        Ref::map(self.data.borrow(), |data| &data.setup_state)
    }

    pub fn set_selected_area(&self, area: Area) {
        self.data.borrow_mut().setup_state.selected_area = area;
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

    pub fn update_player_color(&self, id: PlayerId, color: Color) {
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

        if let Some(mut game) = self.game() {
            let mut players = game.players;
            players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                }
            });
            game.players = players;
            self.data.borrow_mut().game = Some(game)
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

    pub fn show_debug_view(&self) -> bool {
        self.data.borrow().show_debug_view
    }

    #[allow(dead_code)]
    pub fn show_setup_dialog(&self) -> bool {
        self.data.borrow().show_setup_dialog
    }

    #[allow(dead_code)]
    pub fn show_turn_menu(&self) -> bool {
        self.data.borrow().show_turn_menu
    }

    pub fn set_show_turn_menu(&self, show: bool) {
        self.data.borrow_mut().show_turn_menu = show;
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

    pub fn spawn_location(&self, player_id: PlayerId) -> Option<Point> {
        let wave = match self.current_wave() {
            Ok(wave) => wave,
            _ => return None,
        };

        wave.spawn_locations.get(&player_id).cloned()
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

    pub fn erase_mode(&self) -> bool {
        self.data.borrow().erase_mode
    }

    pub fn set_erase_mode(&self, mode: bool) {
        self.data.borrow_mut().erase_mode = mode;
    }
}
