//! リソース管理モジュール
//!
//! ハードウェアや重い初期化を必要とするリソースを管理する。
//! これらはシリアライズ不可であり、AppState とは別に管理される。
//!
//! ## 含まれるリソース
//!
//! - `AssetManager` - 画像テクスチャの読み込み・管理

pub mod asset_manager;

pub use asset_manager::AssetManager;
