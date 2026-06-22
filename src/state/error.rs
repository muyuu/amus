//! 状態ドメインの操作が返すエラー型。

use thiserror::Error;

/// プレイヤーの色変更操作が失敗した理由。
#[derive(Debug, Error)]
pub enum ColorError {
    /// 指定した色を他のプレイヤーが既に使用している。
    #[error("色 {0} は既に使用されています")]
    DuplicateColor(String),
}
