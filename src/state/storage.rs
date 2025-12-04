use eframe::Storage;
use serde::{Deserialize, Serialize};

pub struct AppStorage;

impl AppStorage {
    pub fn get<T>(storage: Option<&dyn Storage>, key: &str) -> Result<Option<T>, String>
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

        match serde_json::from_str::<T>(&json_str) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(format!("'{}' キーでのデシリアライズに失敗: {}", key, e)),
        }
    }

    pub fn set<T>(storage: Option<&mut dyn Storage>, key: &str, value: &T) -> Result<(), String>
    where
        T: Serialize,
    {
        let s = match storage {
            Some(s) => s,
            None => return Ok(()),
        };

        let json_str = match serde_json::to_string(value) {
            Ok(v) => v,
            Err(e) => return Err(format!("'{}' キーでのシリアライズに失敗: {}", key, e)),
        };

        s.set_string(key, json_str);
        s.flush();

        Ok(())
    }

    /// 複数の値を一度に保存するヘルパーメソッド
    pub fn save_multiple<F>(storage: Option<&mut dyn Storage>, save_fn: F) -> Result<(), String>
    where
        F: FnOnce(&mut dyn Storage) -> Result<(), String>,
    {
        match storage {
            Some(s) => save_fn(s),
            None => Ok(()),
        }
    }
}
