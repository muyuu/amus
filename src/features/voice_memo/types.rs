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

/// ラウンドデータ
#[derive(Debug, Clone, Default)]
pub struct Round {
    /// メモ一覧
    pub memos: Vec<VoiceMemo>,
    /// ラウンド終了時の経過秒数
    pub duration_secs: Option<f32>,
}

/// 音声メモの状態
#[derive(Debug, Clone)]
pub struct VoiceMemoState {
    /// 録音中かどうか
    pub is_recording: bool,
    /// 処理中（書き起こし中）かどうか
    pub is_processing: bool,
    /// ラウンドデータ一覧
    pub rounds: Vec<Round>,
    /// 現在表示中のラウンド（0-indexed）
    pub selected_round: usize,
    /// エラーメッセージ
    pub error: Option<String>,
    /// 録音経過時間（秒）
    pub elapsed_secs: f32,
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
    /// ラウンド進行中かどうか
    pub round_active: bool,
    /// ラウンド開始時刻
    pub round_start_time: Option<std::time::Instant>,
    /// ラウンド経過時間（秒）
    pub round_elapsed_secs: f32,
    /// 処理待ちのチャンク数
    pub pending_chunks: usize,
}

impl Default for VoiceMemoState {
    fn default() -> Self {
        Self {
            is_recording: false,
            is_processing: false,
            rounds: Vec::new(),
            selected_round: 0,
            error: None,
            elapsed_secs: 0.0,
            model_available: false,
            recorder_available: false,
            is_downloading: false,
            download_progress: None,
            downloaded_bytes: 0,
            total_bytes: None,
            round_active: false,
            round_start_time: None,
            round_elapsed_secs: 0.0,
            pending_chunks: 0,
        }
    }
}
