//! 汎用ファイルダウンロードリソース
//!
//! 大きなファイルをバックグラウンドでダウンロードし、進捗を報告する。

use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

/// ファイルダウンロードが失敗した理由。
#[derive(Debug, Error)]
pub enum DownloadError {
    /// HTTP リクエストまたは応答の受信に失敗した。
    #[error("ダウンロードのリクエストに失敗: {0}")]
    Request(#[from] ureq::Error),
    /// 保存先のファイル・ディレクトリ操作に失敗した。
    #[error("ファイル操作に失敗: {0}")]
    Io(#[from] std::io::Error),
}

/// ダウンロード進捗
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

impl DownloadProgress {
    pub fn percentage(&self) -> Option<f32> {
        self.total_bytes.map(|total| {
            if total == 0 {
                0.0
            } else {
                (self.downloaded_bytes as f32 / total as f32) * 100.0
            }
        })
    }
}

/// ファイルをダウンロード
///
/// # Arguments
/// * `url` - ダウンロードURL
/// * `dest_path` - 保存先パス
/// * `progress_tx` - 進捗を送信するチャネル
/// * `cancel` - true になったら中断する（アプリ終了時など）
///
/// 中断された場合は `.download` 一時ファイルを削除して `Ok(())` を返す
/// （保存先はリネームされないため未完了のまま残らない）。
pub fn download_file(
    url: &str,
    dest_path: &str,
    progress_tx: std::sync::mpsc::Sender<DownloadProgress>,
    cancel: Arc<AtomicBool>,
) -> Result<(), DownloadError> {
    // ディレクトリを作成
    if let Some(parent) = Path::new(dest_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // ダウンロード開始
    let response = ureq::get(url).call()?;

    let total_bytes = response
        .headers()
        .get("Content-Length")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // 一時ファイルに書き込み
    let temp_path = format!("{}.download", dest_path);
    let mut file = std::fs::File::create(&temp_path)?;

    let mut reader = response.into_body().into_reader();
    let mut buffer = [0u8; 8192];
    let mut downloaded_bytes = 0u64;

    loop {
        if cancel.load(Ordering::Relaxed) {
            // 中断: 開いているファイルを閉じてから一時ファイルを掃除する
            drop(file);
            let _ = std::fs::remove_file(&temp_path);
            return Ok(());
        }

        let bytes_read = reader.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])?;

        downloaded_bytes += bytes_read as u64;

        // 進捗を送信（エラーは無視、受信側が閉じている場合）
        let _ = progress_tx.send(DownloadProgress {
            downloaded_bytes,
            total_bytes,
        });
    }

    // 一時ファイルをリネーム
    std::fs::rename(&temp_path, dest_path)?;

    Ok(())
}
