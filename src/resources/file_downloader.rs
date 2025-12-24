//! 汎用ファイルダウンロードリソース
//!
//! 大きなファイルをバックグラウンドでダウンロードし、進捗を報告する。

use std::io::{Read, Write};
use std::path::Path;

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
///
/// # Returns
/// 成功時は `Ok(())`、失敗時はエラーメッセージ
pub fn download_file(
    url: &str,
    dest_path: &str,
    progress_tx: std::sync::mpsc::Sender<DownloadProgress>,
) -> Result<(), String> {
    // ディレクトリを作成
    if let Some(parent) = Path::new(dest_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成に失敗: {}", e))?;
    }

    // ダウンロード開始
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("ダウンロード開始に失敗: {}", e))?;

    let total_bytes = response
        .headers()
        .get("Content-Length")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // 一時ファイルに書き込み
    let temp_path = format!("{}.download", dest_path);
    let mut file =
        std::fs::File::create(&temp_path).map_err(|e| format!("ファイル作成に失敗: {}", e))?;

    let mut reader = response.into_body().into_reader();
    let mut buffer = [0u8; 8192];
    let mut downloaded_bytes = 0u64;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("読み込みエラー: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("書き込みエラー: {}", e))?;

        downloaded_bytes += bytes_read as u64;

        // 進捗を送信（エラーは無視、受信側が閉じている場合）
        let _ = progress_tx.send(DownloadProgress {
            downloaded_bytes,
            total_bytes,
        });
    }

    // 一時ファイルをリネーム
    std::fs::rename(&temp_path, dest_path).map_err(|e| format!("ファイル移動に失敗: {}", e))?;

    Ok(())
}
