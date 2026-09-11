//! リソース管理モジュール
//!
//! ハードウェアや重い初期化を必要とするリソースを管理する。
//! これらはシリアライズ不可であり、AppState とは別に管理される。
//!
//! ## 含まれるリソース
//!
//! - `AssetManager` - 画像テクスチャの読み込み・管理（egui context経由）
//! - `VoiceRecorder` - マイク音声の録音（ネイティブのみ）
//! - `WhisperTranscriber` - 音声の書き起こし（ネイティブのみ）
//! - `file_downloader` - 大きなファイルのダウンロード（ネイティブのみ）

pub mod asset_manager;

pub use asset_manager::AssetManager;

// 音声メモ関連リソース（ネイティブのみ）
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod file_downloader;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod transcriber_thread;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod voice_recorder;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod resampler;
pub mod whisper_prompt;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod whisper_transcriber;

#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use file_downloader::{download_file, DownloadError, DownloadProgress, IntegrityCheck};
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use transcriber_thread::{TranscribeRequest, TranscriberThread};
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use voice_recorder::VoiceRecorder;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use whisper_transcriber::{
    get_model_download_url, get_model_max_download_bytes, get_model_path, get_model_sha256,
    model_exists, WhisperTranscriber,
};

/// ハードウェアリソース
///
/// シリアライズ不可のハードウェア依存リソースを保持する。
/// Feature から必要に応じて参照される。
pub struct Resources {
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub voice_recorder: Option<VoiceRecorder>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub whisper_transcriber: Option<WhisperTranscriber>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            voice_recorder: Self::init_voice_recorder(),
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            whisper_transcriber: Self::init_whisper_transcriber(),
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    fn init_voice_recorder() -> Option<VoiceRecorder> {
        match VoiceRecorder::new() {
            Ok(r) => Some(r),
            Err(e) => {
                crate::log_error!("Resources", format!("VoiceRecorder初期化エラー: {}", e));
                None
            }
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    fn init_whisper_transcriber() -> Option<WhisperTranscriber> {
        if model_exists() {
            match WhisperTranscriber::new(&get_model_path()) {
                Ok(t) => Some(t),
                Err(e) => {
                    crate::log_error!(
                        "Resources",
                        format!("WhisperTranscriber初期化エラー: {}", e)
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
