//! 汎用ファイルダウンロードリソース
//!
//! 大きなファイルをバックグラウンドでダウンロードし、進捗を報告する。
//! 対外境界（外部から取得したバイナリ）なので、任意で **サイズ上限**と
//! **SHA-256 完全性検証**を掛けられる。検証は一時ファイル（`.download`）へ
//! 書き込みながら行い、成功時のみ本来のパスへ rename する。中断・失敗時は
//! 一時ファイルを削除する。

use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

use sha2::{Digest, Sha256};

/// ファイルダウンロードが失敗した理由。
#[derive(Debug, Error)]
pub enum DownloadError {
    /// HTTP リクエストまたは応答の受信に失敗した。
    #[error("ダウンロードのリクエストに失敗: {0}")]
    Request(#[from] ureq::Error),
    /// 保存先のファイル・ディレクトリ操作に失敗した。
    #[error("ファイル操作に失敗: {0}")]
    Io(#[from] std::io::Error),
    /// 書き込み量が上限を超えた（暴走・肥大化の防止）。
    #[error("ダウンロードサイズが上限 {limit} バイトを超えました")]
    TooLarge { limit: u64 },
    /// ダウンロード結果の SHA-256 が期待値と一致しなかった。
    #[error("チェックサム不一致: expected={expected}, actual={actual}")]
    ChecksumMismatch { expected: String, actual: String },
}

/// ダウンロード完全性チェックの指定（任意）。
#[derive(Debug, Default, Clone)]
pub struct IntegrityCheck<'a> {
    /// 書き込みを打ち切る上限バイト数。`None` なら無制限。
    pub max_bytes: Option<u64>,
    /// 期待する SHA-256（16進小文字）。`None` なら検証しない。
    pub sha256_hex: Option<&'a str>,
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

/// ダウンロードを最後までやり切ったか、途中で中断されたか。
enum Outcome {
    Finished,
    Cancelled,
}

/// ファイルをダウンロード
///
/// # Arguments
/// * `url` - ダウンロードURL
/// * `dest_path` - 保存先パス
/// * `progress_tx` - 進捗を送信するチャネル
/// * `cancel` - true になったら中断する（アプリ終了時など）
/// * `check` - サイズ上限・SHA-256 検証（任意）
///
/// 完全に取得し検証を通った場合のみ `dest_path` へ rename する。中断・エラー
/// （サイズ超過・チェックサム不一致・IO 失敗）時は `.download` 一時ファイルを削除する。
pub fn download_file(
    url: &str,
    dest_path: &str,
    progress_tx: std::sync::mpsc::Sender<DownloadProgress>,
    cancel: Arc<AtomicBool>,
    check: &IntegrityCheck<'_>,
) -> Result<(), DownloadError> {
    // ディレクトリを作成
    if let Some(parent) = Path::new(dest_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let temp_path = format!("{}.download", dest_path);

    match stream_to_temp(url, &temp_path, &progress_tx, &cancel, check) {
        Ok(Outcome::Finished) => {
            // 検証を通った完全なファイルだけを本来のパスへ移す
            if let Err(e) = std::fs::rename(&temp_path, dest_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(e.into());
            }
            Ok(())
        }
        Ok(Outcome::Cancelled) => {
            let _ = std::fs::remove_file(&temp_path);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// 応答を一時ファイルへ流し込みつつ、サイズ上限と SHA-256 を検証する。
fn stream_to_temp(
    url: &str,
    temp_path: &str,
    progress_tx: &std::sync::mpsc::Sender<DownloadProgress>,
    cancel: &Arc<AtomicBool>,
    check: &IntegrityCheck<'_>,
) -> Result<Outcome, DownloadError> {
    let response = ureq::get(url).call()?;

    let total_bytes = response
        .headers()
        .get("Content-Length")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let mut file = std::fs::File::create(temp_path)?;
    let mut reader = response.into_body().into_reader();
    let mut buffer = [0u8; 8192];
    let mut downloaded_bytes = 0u64;
    // sha256_hex 指定時のみハッシュを計算する
    let mut hasher = check.sha256_hex.map(|_| Sha256::new());

    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(Outcome::Cancelled);
        }

        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        downloaded_bytes += bytes_read as u64;
        if let Some(limit) = check.max_bytes {
            if downloaded_bytes > limit {
                // Content-Length を信頼せず、実際の書き込み量で打ち切る
                return Err(DownloadError::TooLarge { limit });
            }
        }

        file.write_all(&buffer[..bytes_read])?;
        if let Some(h) = hasher.as_mut() {
            h.update(&buffer[..bytes_read]);
        }

        // 進捗を送信（エラーは無視、受信側が閉じている場合）
        let _ = progress_tx.send(DownloadProgress {
            downloaded_bytes,
            total_bytes,
        });
    }

    // 完全性検証（rename 前）
    if let (Some(expected), Some(h)) = (check.sha256_hex, hasher) {
        let actual = to_hex(&h.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(DownloadError::ChecksumMismatch {
                expected: expected.to_string(),
                actual,
            });
        }
    }

    Ok(Outcome::Finished)
}

/// バイト列を 16進小文字の文字列にする。
fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_formats_lowercase_zero_padded() {
        assert_eq!(to_hex(&[0x00, 0x0f, 0xa0, 0xff]), "000fa0ff");
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn sha256_of_abc_matches_known_vector() {
        // NIST の既知ベクタ: SHA-256("abc")
        let digest = Sha256::digest(b"abc");
        assert_eq!(
            to_hex(&digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
