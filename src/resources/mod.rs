//! リソース管理モジュール
//!
//! ハードウェアや重い初期化を必要とするリソースを管理する。
//! これらはシリアライズ不可であり、AppState とは別に管理される。
//!
//! ## 含まれるリソース
//!
//! - `AssetManager` - 画像テクスチャの読み込み・管理（egui context経由）
//! - `VoiceRecorder` - マイク音声の録音（ネイティブのみ、`transcribe` クレート）
//! - `WhisperTranscriber` - 音声の書き起こし（ネイティブのみ、`transcribe` クレート）
//! - `file_downloader` - 大きなファイルのダウンロード（ネイティブのみ、`transcribe` クレート）
//!
//! 音声認識まわりは GUI（egui/wgpu）に依存しない別クレート `transcribe` に切り出して
//! ある。モデルや認識パラメータの検証をアプリの再ビルドなしに行うため（`transcribe-check`
//! CLI、詳細は `crates/transcribe`）。ここではそのまま re-export し、既存の呼び出し側
//! （`crate::resources::whisper_transcriber::...` 等）を変えずに済むようにしている。

pub mod asset_manager;

pub use asset_manager::AssetManager;

// 音声メモ関連リソース（ネイティブのみ、実体は `transcribe` クレート）
#[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
pub use transcribe::{
    file_downloader, transcriber_thread, voice_recorder, whisper_backend, whisper_prompt,
    whisper_transcriber,
};

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
    pub fn new() -> Self {
        Self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            voice_recorder: Self::init_voice_recorder(),
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            whisper_transcriber: Self::init_whisper_transcriber(TranscribeSetup::compiled()),
            #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
            transcribe_setup: TranscribeSetup::compiled(),
        }
    }

    /// 書き起こしを用意し直す。
    ///
    /// モデルのダウンロードが終わったときに呼ぶ。まだモデルが無ければ未用意のままにする。
    /// GPU の初期化に失敗した場合は CPU で用意される（構成と一致するとは限らない）。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn reload_transcriber(&mut self) {
        self.whisper_transcriber = Self::init_whisper_transcriber(self.transcribe_setup);
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

    /// GPU で動かすビルドなのに CPU へ退避した場合だけ、実際の構成を返す。
    ///
    /// 構成どおりに動いているなら知らせることはない。GPU 版が遅いときの唯一の説明に
    /// なるため、食い違ったときだけ見せる。
    #[cfg(all(not(target_arch = "wasm32"), feature = "voice_memo"))]
    pub fn transcribe_fallback(&self) -> Option<TranscribeSetup> {
        let actual = self.whisper_transcriber.as_ref()?.setup();
        (actual != self.transcribe_setup).then_some(actual)
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
