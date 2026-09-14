//! Whisper による音声書き起こしエンジン。
//!
//! GUI（egui/wgpu）に依存しない、音声認識部分だけを切り出したクレート。
//! アプリ本体からはリソースとして使う一方、`bin/transcribe_check.rs` から単体でも
//! 動かせるため、モデルや認識パラメータの検証をアプリの再ビルドなしに行える。

#[cfg(feature = "record-audio")]
pub mod debug_recording;
pub mod file_downloader;
pub mod resampler;
pub mod speech;
pub mod transcriber_thread;
pub mod voice_recorder;
pub mod whisper_backend;
pub mod whisper_prompt;
pub mod whisper_transcriber;

pub use file_downloader::{download_file, DownloadError, DownloadProgress, IntegrityCheck};
pub use speech::SpeechDetector;
pub use transcriber_thread::{RecordedSegment, TranscribeRequest, TranscriberThread};
pub use voice_recorder::VoiceRecorder;
pub use whisper_backend::TranscribeSetup;
pub use whisper_transcriber::WhisperTranscriber;

/// ログマクロ。
///
/// アプリ本体（`memongus_wasm`）は独自の構造化ログ（タグ・色付け・wasm対応）を持つが、
/// このクレートはアプリに依存できない（依存の向きが逆）ため、素の `log` クレートに
/// 委譲する簡易版を独自に持つ。呼び出し側の書式（タグ, メッセージ）はアプリ側と揃えて
/// あるので、`crate::log_debug!` の呼び出し自体はアプリのコードと同じ形のまま使える。
#[macro_export]
macro_rules! log_debug {
    ($tag:expr, $message:expr) => {
        log::debug!("[{}] {}", $tag, $message)
    };
}

#[macro_export]
macro_rules! log_error {
    ($tag:expr, $message:expr) => {
        log::error!("[{}] {}", $tag, $message)
    };
}
