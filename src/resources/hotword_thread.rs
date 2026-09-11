//! バックグラウンドホットワード検知スレッド
//!
//! 短い音声窓を繰り返し書き起こす。UI をブロックしないよう専用スレッドで実行する。
//! 検知用は速度を優先して tiny モデルを使い、書き起こし用とは別のモデルを持つ。
//!
//! 書き起こすところまでがこのスレッドの責務で、トリガーワードとの照合は呼び出し側が行う。

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use thiserror::Error;

use super::whisper_transcriber::{WhisperModel, WhisperTranscriber};
use crate::{log_debug, log_error};

/// 検知スレッドの起動が失敗した理由。
#[derive(Debug, Error)]
pub enum HotwordThreadError {
    #[error("Whisper モデルが見つかりません: {0}")]
    ModelNotFound(String),
}

/// 検知リクエスト
pub struct HotwordRequest {
    /// 音声窓（16kHz, mono, f32）
    pub samples: Vec<f32>,
    /// 窓の終端の絶対サンプルインデックス。検知したときのターン境界に使う。
    pub window_end_sample: usize,
}

/// 検知結果
#[derive(Debug, Clone)]
pub struct HotwordResult {
    /// 窓を書き起こしたテキスト。照合は呼び出し側で行う。
    pub text: String,
    /// 窓の終端の絶対サンプルインデックス
    pub window_end_sample: usize,
}

/// バックグラウンドホットワード検知スレッド
pub struct HotwordThread {
    /// 送信側。drop で `recv` を終了させたいので `Option` で持ち、`Drop` で先に落とす。
    request_tx: Option<Sender<HotwordRequest>>,
    result_rx: Receiver<HotwordResult>,
    thread_handle: Option<JoinHandle<()>>,
}

impl HotwordThread {
    pub fn new() -> Result<Self, HotwordThreadError> {
        if !WhisperModel::TINY.exists() {
            return Err(HotwordThreadError::ModelNotFound(WhisperModel::TINY.path()));
        }

        let (request_tx, request_rx) = mpsc::channel::<HotwordRequest>();
        let (result_tx, result_rx) = mpsc::channel::<HotwordResult>();

        let thread_handle = thread::spawn(move || {
            Self::detector_loop(request_rx, result_tx);
        });

        Ok(Self {
            request_tx: Some(request_tx),
            result_rx,
            thread_handle: Some(thread_handle),
        })
    }

    /// 検知リクエストを送る。
    pub fn request(&self, req: HotwordRequest) {
        if let Some(tx) = &self.request_tx {
            let _ = tx.send(req);
        }
    }

    /// 結果をポーリングする（非ブロッキング）。
    pub fn poll_results(&self) -> Vec<HotwordResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.result_rx.try_recv() {
            results.push(result);
        }
        results
    }

    fn detector_loop(request_rx: Receiver<HotwordRequest>, result_tx: Sender<HotwordResult>) {
        let transcriber = match WhisperTranscriber::new(&WhisperModel::TINY.path()) {
            Ok(t) => t,
            Err(e) => {
                log_error!("HotwordThread", format!("Whisper初期化エラー: {}", e));
                return;
            }
        };

        log_debug!("HotwordThread", "検知スレッド開始");

        loop {
            match request_rx.recv() {
                Ok(req) => {
                    // 検知は語句が拾えればよいので認識コンテキストは渡さない。
                    let text = match transcriber.transcribe(&req.samples, None) {
                        Ok(segments) => segments
                            .into_iter()
                            .map(|s| s.text)
                            .collect::<Vec<_>>()
                            .join(""),
                        Err(e) => {
                            log_error!("HotwordThread", format!("検知エラー: {}", e));
                            continue;
                        }
                    };

                    let _ = result_tx.send(HotwordResult {
                        text,
                        window_end_sample: req.window_end_sample,
                    });
                }
                Err(_) => {
                    log_debug!("HotwordThread", "検知スレッド終了");
                    break;
                }
            }
        }
    }
}

impl Drop for HotwordThread {
    /// 送信側を先に落として `recv` を終了させ、スレッドの完了を待ってから抜ける。
    fn drop(&mut self) {
        self.request_tx.take();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
