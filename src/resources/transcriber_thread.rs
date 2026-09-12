//! バックグラウンド書き起こしスレッド
//!
//! Whisper書き起こしを専用スレッドで実行し、UIをブロックしない。

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use thiserror::Error;

use super::voice_recorder::SAMPLE_RATE;
use super::whisper_transcriber::{TranscriptionSegment, WhisperModel};
use crate::{log_debug, log_error};

/// 書き起こしスレッドの起動が失敗した理由。
#[derive(Debug, Error)]
pub enum TranscriberThreadError {
    #[error("Whisper モデルが見つかりません: {0}")]
    ModelNotFound(String),
}

/// 書き起こしリクエスト
pub struct TranscribeRequest {
    /// 音声サンプル（16kHz, mono, f32）
    pub samples: Vec<f32>,
    /// サンプル列の先頭の絶対サンプル位置
    pub start_sample: usize,
    /// 認識コンテキスト
    pub context: Option<String>,
}

/// 録音上の位置を持つ書き起こしセグメント。
///
/// 位置は `SAMPLE_RATE` 基準の絶対サンプルインデックス。ターンの開閉は書き起こしより
/// 後に決まりうるため、所属ではなく位置を持たせて呼び出し側に判断させる。
#[derive(Debug, Clone)]
pub struct RecordedSegment {
    /// セグメント開始位置
    pub start_sample: usize,
    /// セグメント終了位置
    pub end_sample: usize,
    /// 書き起こしテキスト
    pub text: String,
}

/// 書き起こし結果
#[derive(Debug, Clone)]
pub struct TranscribeResult {
    /// 書き起こしセグメント
    pub segments: Vec<RecordedSegment>,
    /// エラーメッセージ（あれば）
    pub error: Option<String>,
}

/// バックグラウンド書き起こしスレッド
pub struct TranscriberThread {
    /// 送信側。drop で `recv` を終了させたいので `Option` で持ち、`Drop` で先に落とす。
    request_tx: Option<Sender<TranscribeRequest>>,
    result_rx: Receiver<TranscribeResult>,
    thread_handle: Option<JoinHandle<()>>,
}

impl TranscriberThread {
    /// 新しいTranscriberThreadを作成
    pub fn new() -> Result<Self, TranscriberThreadError> {
        if !WhisperModel::SMALL.exists() {
            return Err(TranscriberThreadError::ModelNotFound(
                WhisperModel::SMALL.path(),
            ));
        }

        let (request_tx, request_rx) = mpsc::channel::<TranscribeRequest>();
        let (result_tx, result_rx) = mpsc::channel::<TranscribeResult>();

        let thread_handle = thread::spawn(move || {
            Self::transcriber_loop(request_rx, result_tx);
        });

        Ok(Self {
            request_tx: Some(request_tx),
            result_rx,
            thread_handle: Some(thread_handle),
        })
    }

    /// 書き起こしリクエストを送信
    pub fn request(&self, req: TranscribeRequest) {
        if let Some(tx) = &self.request_tx {
            let _ = tx.send(req);
        }
    }

    /// 結果をポーリング（非ブロッキング）
    pub fn poll_results(&self) -> Vec<TranscribeResult> {
        let mut results = Vec::new();
        loop {
            match self.result_rx.try_recv() {
                Ok(result) => results.push(result),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        results
    }

    /// チャンク相対の秒を録音上の絶対サンプル位置へ直す。
    fn to_recorded(
        segments: Vec<TranscriptionSegment>,
        start_sample: usize,
    ) -> Vec<RecordedSegment> {
        let to_sample = |secs: f32| start_sample + (secs * SAMPLE_RATE as f32) as usize;

        segments
            .into_iter()
            .map(|seg| RecordedSegment {
                start_sample: to_sample(seg.start_secs),
                end_sample: to_sample(seg.end_secs),
                text: seg.text,
            })
            .collect()
    }

    /// 書き起こしループ（バックグラウンドスレッド）
    fn transcriber_loop(
        request_rx: Receiver<TranscribeRequest>,
        result_tx: Sender<TranscribeResult>,
    ) {
        use super::whisper_transcriber::WhisperTranscriber;

        // Whisperを初期化
        let mut transcriber = match WhisperTranscriber::new(&WhisperModel::SMALL.path()) {
            Ok(t) => t,
            Err(e) => {
                log_error!("TranscriberThread", format!("Whisper初期化エラー: {}", e));
                return;
            }
        };

        log_debug!(
            "TranscriberThread",
            format!(
                "書き起こしスレッド開始: {}スレッド",
                super::whisper_transcriber::thread_count()
            )
        );

        loop {
            // リクエストを待機（ブロッキング）
            match request_rx.recv() {
                Ok(req) => {
                    let samples_len = req.samples.len();
                    let duration_secs = samples_len as f32 / SAMPLE_RATE as f32;
                    log_debug!(
                        "TranscriberThread",
                        format!(
                            "書き起こし開始: {:.1}秒分 ({}サンプル), start_sample={}",
                            duration_secs, samples_len, req.start_sample
                        )
                    );

                    let started = std::time::Instant::now();
                    let transcribed = transcriber.transcribe(&req.samples, req.context.as_deref());
                    let elapsed = started.elapsed().as_secs_f32();

                    let result = match transcribed {
                        Ok(segments) => {
                            let segments = Self::to_recorded(segments, req.start_sample);
                            // 書き起こし全文は発話内容そのものなのでログに残さない（プライバシー）。
                            // 件数と処理時間のみ記録する。
                            //
                            // RTF（処理時間 / 音声長）が 1 を超えると入力に追いつかず、
                            // 待ち行列が伸び続ける。スレッド数を決める指標。
                            log_debug!(
                                "TranscriberThread",
                                format!(
                                    "書き起こし完了: {}セグメント, {:.2}秒 (RTF={:.2})",
                                    segments.len(),
                                    elapsed,
                                    elapsed / duration_secs.max(f32::EPSILON)
                                )
                            );
                            TranscribeResult {
                                segments,
                                error: None,
                            }
                        }
                        Err(e) => {
                            log_error!("TranscriberThread", format!("書き起こしエラー: {}", e));
                            TranscribeResult {
                                segments: Vec::new(),
                                error: Some(e.to_string()),
                            }
                        }
                    };

                    let _ = result_tx.send(result);
                }
                Err(_) => {
                    // チャネルが閉じられた
                    log_debug!("TranscriberThread", "書き起こしスレッド終了");
                    break;
                }
            }
        }
    }
}

impl Drop for TranscriberThread {
    /// 送信側を先に落として `recv` を終了させ、スレッドの完了を待ってから抜ける。
    /// 書き起こし中に drop された場合は、そのチャンクの完了まで join でブロックする。
    fn drop(&mut self) {
        self.request_tx.take(); // Sender を drop → transcriber_loop の recv が Err → 終了
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
