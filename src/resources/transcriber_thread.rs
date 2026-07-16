//! バックグラウンド書き起こしスレッド
//!
//! Whisper書き起こしを専用スレッドで実行し、UIをブロックしない。

use chrono::Local;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use thiserror::Error;

use super::whisper_transcriber::{get_model_path, model_exists, TranscriptionSegment};

/// 書き起こしスレッドの起動が失敗した理由。
#[derive(Debug, Error)]
pub enum TranscriberThreadError {
    #[error("Whisper モデルが見つかりません: {0}")]
    ModelNotFound(String),
}

fn now() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

/// 書き起こしリクエスト
pub struct TranscribeRequest {
    /// 音声サンプル（16kHz, mono, f32）
    pub samples: Vec<f32>,
    /// 録音開始からのオフセット（秒）
    pub offset_secs: f32,
    /// 認識コンテキスト
    pub context: Option<String>,
    /// どのラウンドか
    pub round_index: usize,
}

/// 書き起こし結果
#[derive(Debug, Clone)]
pub struct TranscribeResult {
    /// 書き起こしセグメント
    pub segments: Vec<TranscriptionSegment>,
    /// どのラウンドか
    pub round_index: usize,
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
        if !model_exists() {
            return Err(TranscriberThreadError::ModelNotFound(get_model_path()));
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

    /// 書き起こしループ（バックグラウンドスレッド）
    fn transcriber_loop(
        request_rx: Receiver<TranscribeRequest>,
        result_tx: Sender<TranscribeResult>,
    ) {
        use super::whisper_transcriber::WhisperTranscriber;

        // Whisperを初期化
        let transcriber = match WhisperTranscriber::new(&get_model_path()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{} [TranscriberThread] Whisper初期化エラー: {}", now(), e);
                return;
            }
        };

        eprintln!("{} [TranscriberThread] 書き起こしスレッド開始", now());

        loop {
            // リクエストを待機（ブロッキング）
            match request_rx.recv() {
                Ok(req) => {
                    let samples_len = req.samples.len();
                    let duration_secs = samples_len as f32 / 16000.0;
                    eprintln!(
                        "{} [TranscriberThread] 書き起こし開始: {:.1}秒分 ({}サンプル), offset={:.1}s, round={}",
                        now(),
                        duration_secs,
                        samples_len,
                        req.offset_secs,
                        req.round_index
                    );

                    let result = match transcriber.transcribe(&req.samples, req.context.as_deref())
                    {
                        Ok(mut segments) => {
                            // オフセットを加算
                            for seg in &mut segments {
                                seg.timestamp_secs += req.offset_secs;
                            }
                            // 結果のテキストも表示
                            let texts: Vec<&str> =
                                segments.iter().map(|s| s.text.as_str()).collect();
                            eprintln!(
                                "{} [TranscriberThread] 書き起こし完了: {}セグメント {:?}",
                                now(),
                                segments.len(),
                                texts
                            );
                            TranscribeResult {
                                segments,
                                round_index: req.round_index,
                                error: None,
                            }
                        }
                        Err(e) => {
                            eprintln!("{} [TranscriberThread] 書き起こしエラー: {}", now(), e);
                            TranscribeResult {
                                segments: Vec::new(),
                                round_index: req.round_index,
                                error: Some(e.to_string()),
                            }
                        }
                    };

                    let _ = result_tx.send(result);
                }
                Err(_) => {
                    // チャネルが閉じられた
                    eprintln!("{} [TranscriberThread] 書き起こしスレッド終了", now());
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
