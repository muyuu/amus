use crate::log_debug;
use crate::models::{Game, SetupState};
use crate::state::storage::AppStorage;
use crate::state::storage_keys::StorageKeys;

use super::AppState;

// ストレージ関連
impl AppState {
    /// ストレージからアプリケーション状態を復元
    pub fn load_from_storage(
        &mut self,
        storage: Option<&dyn eframe::Storage>,
    ) -> Result<(), String> {
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
        AppStorage::set(storage, key, value)
    }

    #[allow(dead_code)]
    /// ゲームデータのみをクリア（設定は保持）
    pub fn clear_game_data(
        &mut self,
        storage: Option<&mut dyn eframe::Storage>,
    ) -> Result<(), String> {
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
