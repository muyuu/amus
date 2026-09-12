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
pub mod resampler;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod transcriber_thread;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod voice_recorder;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod whisper_backend;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod whisper_prompt;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub mod whisper_transcriber;

#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use file_downloader::{download_file, DownloadError, DownloadProgress, IntegrityCheck};
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use transcriber_thread::{RecordedSegment, TranscribeRequest, TranscriberThread};
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use voice_recorder::VoiceRecorder;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use whisper_backend::TranscribeSetup;
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use whisper_transcriber::WhisperTranscriber;

/// ハードウェアリソース
///
/// シリアライズ不可のハードウェア依存リソースを保持する。
/// Feature から必要に応じて参照される。
pub struct Resources {
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub voice_recorder: Option<VoiceRecorder>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub whisper_transcriber: Option<WhisperTranscriber>,

    /// 書き起こしに使う構成。モデルが未取得でも決まっているため、`whisper_transcriber`
    /// とは別に持つ。ダウンロードすべきモデルはここから決まる。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    transcribe_setup: TranscribeSetup,
}

impl Resources {
    /// 書き起こしは構成が決まるまで用意しない。保存された設定を読んだあとに
    /// [`Self::reload_transcriber`] を呼ぶこと。
    pub fn new() -> Self {
        Self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            voice_recorder: Self::init_voice_recorder(),
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            whisper_transcriber: None,
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            transcribe_setup: TranscribeSetup::CPU,
        }
    }

    /// 書き起こしを指定の構成で用意し直す。
    ///
    /// 起動時に保存された設定を反映するとき、設定が切り替わったとき、モデルの
    /// ダウンロードが終わったときに呼ぶ。モデルファイルがまだ無ければ未用意のままにする。
    /// GPU の初期化に失敗した場合は CPU で用意される（要求した構成と一致するとは限らない）。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn reload_transcriber(&mut self, setup: TranscribeSetup) {
        self.transcribe_setup = setup;
        self.whisper_transcriber = Self::init_whisper_transcriber(setup);
    }

    /// ハードウェアを一切持たない Resources。
    ///
    /// 実機に触れずに Feature の振る舞いを確かめるために使う。
    #[cfg(all(test, not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn without_hardware() -> Self {
        Self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            voice_recorder: None,
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            whisper_transcriber: None,
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            transcribe_setup: TranscribeSetup::CPU,
        }
    }

    /// 書き起こしに使う構成。モデルの取得状況によらず決まっている。
    ///
    /// 実際に動いている構成は `whisper_transcriber` 側が持つ（GPU の初期化に失敗すると
    /// CPU に落ちるため、要求と一致するとは限らない）。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn transcribe_setup(&self) -> TranscribeSetup {
        self.transcribe_setup
    }

    /// 実際に動いている構成の表記。まだ用意できていなければ要求した構成を返す。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn transcribe_label(&self) -> String {
        self.whisper_transcriber
            .as_ref()
            .map(|t| t.setup())
            .unwrap_or(self.transcribe_setup)
            .label()
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
    fn init_whisper_transcriber(setup: TranscribeSetup) -> Option<WhisperTranscriber> {
        if !setup.model.exists() {
            return None;
        }

        match WhisperTranscriber::new(setup) {
            Ok(t) => Some(t),
            Err(e) => {
                crate::log_error!(
                    "Resources",
                    format!("WhisperTranscriber初期化エラー: {}", e)
                );
                None
            }
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
