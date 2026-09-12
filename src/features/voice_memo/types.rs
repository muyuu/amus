//! 音声メモのデータ型（DTO・View 用状態）

use serde::{Deserialize, Serialize};

/// 書き起こされた音声メモ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMemo {
    /// ターン開始からの秒数
    pub timestamp_secs: f32,
    /// 書き起こしテキスト
    pub text: String,
}

/// ターンデータ
///
/// ターンは録音上の時間区間として表す。位置は `VoiceRecorder::SAMPLE_RATE` 基準の
/// 絶対サンプルインデックスで、録音の開始・停止とは独立している。
#[derive(Debug, Clone, Default)]
pub struct Round {
    /// メモ一覧
    pub memos: Vec<VoiceMemo>,
    /// ターン開始位置
    pub start_sample: usize,
    /// ターン終了位置。進行中は `None`。
    pub end_sample: Option<usize>,
}

/// 音声メモの状態
#[derive(Debug, Clone)]
pub struct VoiceMemoState {
    /// 録音中かどうか
    pub is_recording: bool,
    /// 処理中（書き起こし中）かどうか
    pub is_processing: bool,
    /// ターンデータ一覧
    pub rounds: Vec<Round>,
    /// 現在表示中のターン（0-indexed）
    pub selected_round: usize,
    /// エラーメッセージ
    pub error: Option<String>,
    /// Whisperモデルが利用可能か
    pub model_available: bool,
    /// レコーダーが利用可能か
    pub recorder_available: bool,
    /// モデルダウンロード中かどうか
    pub is_downloading: bool,
    /// ダウンロード進捗（0.0〜100.0）
    pub download_progress: Option<f32>,
    /// ダウンロード済みバイト数
    pub downloaded_bytes: u64,
    /// ダウンロード総バイト数
    pub total_bytes: Option<u64>,
    /// ターン進行中かどうか
    pub round_active: bool,
    /// ターン経過時間（秒）
    pub round_elapsed_secs: f32,
    /// 処理待ちのチャンク数
    pub pending_chunks: usize,

    /// 動作中の構成（`Vulkan / medium` など）
    pub backend_label: String,
    /// 必要なモデルの目安サイズ表記（ダウンロードを促すときに出す）
    pub model_size_label: String,
}

impl Default for VoiceMemoState {
    fn default() -> Self {
        Self {
            is_recording: false,
            is_processing: false,
            rounds: Vec::new(),
            selected_round: 0,
            error: None,
            model_available: false,
            recorder_available: false,
            is_downloading: false,
            download_progress: None,
            downloaded_bytes: 0,
            total_bytes: None,
            round_active: false,
            round_elapsed_secs: 0.0,
            pending_chunks: 0,
            backend_label: String::new(),
            model_size_label: String::new(),
        }
    }
}
