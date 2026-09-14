use crate::log_debug;
use crate::models::{Game, SetupState};
use crate::state::storage::{AppStorage, StorageError};
use crate::state::storage_keys::StorageKeys;

use super::AppState;

// ストレージ関連
impl AppState {
    /// ストレージからアプリケーション状態を復元
    pub fn load_from_storage(
        &mut self,
        storage: Option<&dyn eframe::Storage>,
    ) -> Result<(), StorageError> {
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

        // UI 拡大率（保存値は Option<f32>: None = 自動）
        if let Ok(Some(ui_scale)) = AppStorage::get::<Option<f32>>(storage, StorageKeys::UI_SCALE) {
            self.data.ui_scale = ui_scale;
        }

        // 軌跡描画の線の太さ
        if let Ok(Some(route_line_width)) =
            AppStorage::get::<f32>(storage, StorageKeys::ROUTE_LINE_WIDTH)
        {
            self.data.route_line_width = route_line_width;
        }

        // 録音に使う入力デバイス名（保存値は Option<String>: None = システム既定）
        #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
        if let Ok(Some(input_device_name)) =
            AppStorage::get::<Option<String>>(storage, StorageKeys::INPUT_DEVICE_NAME)
        {
            self.data.input_device_name = input_device_name;
        }

        Ok(())
    }

    /// アプリケーション状態をストレージに保存
    pub fn save_to_storage(
        &self,
        storage: Option<&mut dyn eframe::Storage>,
    ) -> Result<(), StorageError> {
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
            AppStorage::set(
                Some(s),
                StorageKeys::SHOW_DEBUG_VIEW,
                &self.data.show_debug_view,
            )?;
            AppStorage::set(Some(s), StorageKeys::ERASE_MODE, &self.data.erase_mode)?;
            AppStorage::set(Some(s), StorageKeys::UI_SCALE, &self.data.ui_scale)?;
            AppStorage::set(
                Some(s),
                StorageKeys::ROUTE_LINE_WIDTH,
                &self.data.route_line_width,
            )?;
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            AppStorage::set(
                Some(s),
                StorageKeys::INPUT_DEVICE_NAME,
                &self.data.input_device_name,
            )?;

            Ok(())
        })
    }

    #[allow(dead_code)]
    /// ストレージから特定のキーのデータを読み込み
    pub fn load_value<T>(
        &self,
        storage: Option<&dyn eframe::Storage>,
        key: &str,
    ) -> Result<Option<T>, StorageError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        AppStorage::get(storage, key)
    }

    #[allow(dead_code)]
    /// ストレージに特定のキーでデータを保存
    pub fn save_value<T>(
        &self,
        storage: Option<&mut dyn eframe::Storage>,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError>
    where
        T: serde::Serialize,
    {
        AppStorage::set(storage, key, value)
    }

    #[allow(dead_code)]
    /// ゲームデータのみをクリア（設定は保持）
    pub fn clear_game_data(
        &mut self,
        storage: Option<&mut dyn eframe::Storage>,
    ) -> Result<(), StorageError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Area;
    use std::collections::HashMap;

    /// テスト用のインメモリ eframe::Storage 実装
    #[derive(Default)]
    struct MemStorage {
        map: HashMap<String, String>,
    }

    impl eframe::Storage for MemStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.map.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.map.insert(key.to_string(), value);
        }
        fn flush(&mut self) {}
    }

    #[test]
    fn save_then_load_round_trips_game_and_ui_state() {
        let mut storage = MemStorage::default();

        // 保存元の状態を作る
        let mut src = AppState::new();
        src.set_selected_area(Area::Polus);
        src.create_game_from_setup();
        src.select_wave(4);
        src.set_erase_mode(true);
        src.set_ui_scale(2.0);
        let player_count = src.slices().player().players().unwrap().len();

        src.save_to_storage(Some(&mut storage)).unwrap();

        // 別の AppState に読み戻す
        let mut dst = AppState::new();
        dst.load_from_storage(Some(&storage)).unwrap();

        assert_eq!(dst.slices().wave().current_wave_index(), 4);
        assert!(dst.slices().ui().erase_mode());
        assert_eq!(dst.ui_scale(), Some(2.0));
        assert_eq!(dst.slices().setup().selected_area(), &Area::Polus);
        let slices = dst.slices();
        let game = slices.game().game().expect("ゲームが復元される");
        assert_eq!(game.players.len(), player_count);
        assert_eq!(game.area, Area::Polus);
    }

    #[test]
    fn load_from_none_storage_keeps_defaults() {
        let mut state = AppState::new();

        state.load_from_storage(None).unwrap();

        // ストレージが無ければ既定値のまま
        assert!(state.slices().game().game().is_none());
        assert_eq!(state.slices().wave().current_wave_index(), 0);
        assert!(!state.slices().ui().erase_mode());
    }
}
