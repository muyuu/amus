use eframe::Storage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ストレージの読み書きが失敗した理由。
#[derive(Debug, Error)]
pub enum StorageError {
    /// 値のシリアライズ（JSON 化）に失敗した。
    #[error("'{key}' キーのシリアライズに失敗: {source}")]
    Serialize {
        key: String,
        source: serde_json::Error,
    },
    /// 保存済み JSON のデシリアライズに失敗した。
    #[error("'{key}' キーのデシリアライズに失敗: {source}")]
    Deserialize {
        key: String,
        source: serde_json::Error,
    },
}

pub struct AppStorage;

impl AppStorage {
    pub fn get<T>(storage: Option<&dyn Storage>, key: &str) -> Result<Option<T>, StorageError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let s = match storage {
            Some(s) => s,
            None => return Ok(None),
        };

        let json_str = match s.get_string(key) {
            Some(v) => v,
            None => return Ok(None),
        };

        serde_json::from_str::<T>(&json_str)
            .map(Some)
            .map_err(|source| StorageError::Deserialize {
                key: key.to_string(),
                source,
            })
    }

    pub fn set<T>(
        storage: Option<&mut dyn Storage>,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        let s = match storage {
            Some(s) => s,
            None => return Ok(()),
        };

        let json_str = serde_json::to_string(value).map_err(|source| StorageError::Serialize {
            key: key.to_string(),
            source,
        })?;

        s.set_string(key, json_str);
        s.flush();

        Ok(())
    }

    /// 複数の値を一度に保存するヘルパーメソッド
    pub fn save_multiple<F>(
        storage: Option<&mut dyn Storage>,
        save_fn: F,
    ) -> Result<(), StorageError>
    where
        F: FnOnce(&mut dyn Storage) -> Result<(), StorageError>,
    {
        match storage {
            Some(s) => save_fn(s),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MemStorage {
        map: HashMap<String, String>,
    }

    impl Storage for MemStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.map.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.map.insert(key.to_string(), value);
        }
        fn flush(&mut self) {}
    }

    #[test]
    fn get_returns_deserialize_error_on_corrupt_json() {
        let mut storage = MemStorage::default();
        storage.set_string("k", "not a number".to_string());

        let result = AppStorage::get::<usize>(Some(&storage), "k");

        assert!(matches!(
            result,
            Err(StorageError::Deserialize { ref key, .. }) if key == "k"
        ));
    }
}
