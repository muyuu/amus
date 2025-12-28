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
#[cfg(not(target_arch = "wasm32"))]
pub mod file_downloader;
#[cfg(not(target_arch = "wasm32"))]
pub mod transcriber_thread;
#[cfg(not(target_arch = "wasm32"))]
pub mod voice_recorder;
#[cfg(not(target_arch = "wasm32"))]
pub mod whisper_transcriber;

#[cfg(not(target_arch = "wasm32"))]
pub use file_downloader::{download_file, DownloadProgress};
#[cfg(not(target_arch = "wasm32"))]
pub use voice_recorder::{RecordedSegment, VoiceRecorder};
#[cfg(not(target_arch = "wasm32"))]
pub use whisper_transcriber::{
    get_model_download_url, get_model_path, get_model_size_description, model_exists,
    TranscriptionSegment, WhisperTranscriber,
};
#[cfg(not(target_arch = "wasm32"))]
pub use transcriber_thread::{TranscribeRequest, TranscribeResult, TranscriberThread};

/// ハードウェアリソース
///
/// シリアライズ不可のハードウェア依存リソースを保持する。
/// Feature から必要に応じて参照される。
pub struct Resources {
    #[cfg(not(target_arch = "wasm32"))]
    pub voice_recorder: Option<VoiceRecorder>,
    #[cfg(not(target_arch = "wasm32"))]
    pub whisper_transcriber: Option<WhisperTranscriber>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            voice_recorder: Self::init_voice_recorder(),
            #[cfg(not(target_arch = "wasm32"))]
            whisper_transcriber: Self::init_whisper_transcriber(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn init_voice_recorder() -> Option<VoiceRecorder> {
        match VoiceRecorder::new() {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("VoiceRecorder初期化エラー: {}", e);
                None
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn init_whisper_transcriber() -> Option<WhisperTranscriber> {
        if model_exists() {
            match WhisperTranscriber::new(&get_model_path()) {
                Ok(t) => Some(t),
                Err(e) => {
                    eprintln!("WhisperTranscriber初期化エラー: {}", e);
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
