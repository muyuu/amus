//! 音声メモ機能
//!
//! 音声を録音して Whisper で書き起こす機能（ネイティブ専用）。
//!
//! 責務はファイルに分割している:
//! - `types`: DTO / View 用状態
//! - `vad`: 発話区間検出
//! - `transcription`: 書き起こしスレッドとのやり取り
//! - `download`: Whisper モデルのダウンロード制御
//! - `recording`: ラウンド・録音のライフサイクル
//! - `view`: 描画

mod download;
mod prompt;
mod recording;
mod transcription;
mod types;
mod vad;
pub mod view;

pub use types::{Round, VoiceMemo, VoiceMemoState};

use crate::app_action::AppAction;
use crate::i18n::keys as K;
use crate::i18n::words::ja::JapaneseWords;
use crate::log_error;
use crate::models::Area;
use crate::resources::{DownloadError, DownloadProgress, Resources, TranscriberThread};
use crate::state::Slices;

/// 音声メモのアクション
#[derive(Debug, Clone)]
pub enum VoiceMemoAction {
    StartRound,
    EndRound,
    StartRecording,
    StopRecording,
    SelectRound(usize),
    ClearAllRounds,
    DownloadModel,
}

/// 音声メモ機能
pub struct VoiceMemoFeature {
    /// View用の状態
    pub state: VoiceMemoState,
    /// ダウンロード進捗受信用
    download_rx: Option<std::sync::mpsc::Receiver<DownloadProgress>>,
    /// ダウンロードスレッドハンドル
    download_handle: Option<std::thread::JoinHandle<Result<(), DownloadError>>>,
    /// ダウンロード中断フラグ（アプリ終了時に立てる）
    download_cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// 認識用コンテキスト（プレイヤー名など）
    context: Option<String>,
    /// 描画時に集めた最新の語彙
    vocabulary: Option<prompt::Vocabulary>,
    /// `context` を組み立てた元の語彙。変化したときだけ組み直す。
    context_vocabulary: Option<prompt::Vocabulary>,
    /// バックグラウンド書き起こしスレッド
    transcriber_thread: Option<TranscriberThread>,
    /// 発話開始位置（元サンプルレート基準）
    speech_start_sample: usize,
    /// 最後にVADチェックしたサンプル位置
    last_vad_check_sample: usize,
    /// 無音開始時刻
    silence_start: Option<std::time::Instant>,
    /// 現在発話中かどうか
    is_speaking: bool,
    /// 録音開始時のラウンド経過時間（オフセット計算用）
    recording_start_round_secs: f32,
}

impl VoiceMemoFeature {
    pub fn new(resources: &Resources) -> Self {
        let model_available = resources.whisper_transcriber.is_some();
        let state = VoiceMemoState {
            model_available,
            recorder_available: resources.voice_recorder.is_some(),
            ..Default::default()
        };

        // モデルが利用可能ならTranscriberThreadを初期化
        let transcriber_thread = if model_available {
            match TranscriberThread::new() {
                Ok(t) => Some(t),
                Err(e) => {
                    log_error!(
                        "VoiceMemo",
                        &format!("TranscriberThread初期化エラー: {}", e)
                    );
                    None
                }
            }
        } else {
            None
        };

        Self {
            state,
            download_rx: None,
            download_handle: None,
            download_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            context: None,
            vocabulary: None,
            context_vocabulary: None,
            transcriber_thread,
            speech_start_sample: 0,
            last_vad_check_sample: 0,
            silence_start: None,
            is_speaking: false,
            recording_start_round_secs: 0.0,
        }
    }

    /// 音声メモウィンドウを描画し、操作を Action として返す（中央 dispatch の②）。
    /// 状態のリアルタイム更新は `update()`（①）、Action の適用は `handle_action()`（③）。
    pub fn render(&mut self, slices: &Slices, ui: &mut egui::Ui) -> Vec<AppAction> {
        // 1. コンテキスト情報を収集（Slices から）
        let player_info: Vec<(String, String)> = slices
            .player()
            .players()
            .map(|players| {
                players
                    .iter()
                    .map(|p| (p.name.clone(), p.color.name_ja().to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let room_names: &[&str] = match slices.game().area() {
            Some(Area::Skeld) => JapaneseWords::SKELD_ROOMS,
            Some(Area::Mira) => JapaneseWords::MIRA_ROOMS,
            Some(Area::Polus) => JapaneseWords::POLUS_ROOMS,
            Some(Area::AirShip) | None => JapaneseWords::AIRSHIP_ROOMS,
        };

        // 2. 認識語彙を記録（プロンプトの組み立ては update 側）
        self.set_vocabulary(&player_info, room_names);

        // 3. Viewを描画して Action を取得
        let translator = slices.translator();
        let window_title = translator.t(K::VOICE_MEMO_TITLE);
        let actions = egui::Window::new(window_title)
            .collapsible(true)
            .resizable(true)
            .default_size([300.0, 400.0])
            .show(ui.ctx(), |ui| {
                view::VoiceMemoView::render(&self.state, translator, ui)
            })
            .and_then(|r| r.inner)
            .unwrap_or_default();

        actions.into_iter().map(AppAction::VoiceMemo).collect()
    }

    /// 語彙が変化していれば認識コンテキストを組み直す。
    ///
    /// 組み立てにはモデルのトークナイザが要るため、`WhisperTranscriber` が
    /// 利用できないうちはコンテキストなしで書き起こす。
    fn refresh_context(&mut self, resources: &Resources) {
        let Some(vocabulary) = &self.vocabulary else {
            return;
        };
        if self.context_vocabulary.as_ref() == Some(vocabulary) {
            return;
        }
        let Some(transcriber) = &resources.whisper_transcriber else {
            return;
        };

        self.context = Some(prompt::build_recognition_context(vocabulary, |text| {
            transcriber.count_tokens(text)
        }));
        self.context_vocabulary = Some(vocabulary.clone());
    }

    /// 認識に使う語彙を記録する。プロンプトの組み立ては `refresh_context` が行う。
    ///
    /// 色名はプロンプトに載せない。10人で40トークンを占める一方、名前の認識には
    /// 寄与しないため。
    fn set_vocabulary(
        &mut self,
        players: &[(String, String)],
        room_names: &'static [&'static str],
    ) {
        self.vocabulary = Some(prompt::Vocabulary {
            player_names: players
                .iter()
                .map(|(name, _)| name.clone())
                .filter(|name| !name.is_empty())
                .collect(),
            room_names,
        });
    }

    /// リアルタイム更新（中央 dispatch の①）。タイマー・ポーリング・VAD・ダウンロード進捗。
    pub fn update(&mut self, resources: &Resources, ctx: &egui::Context) {
        // ラウンド進行中ならタイマーを更新
        if self.state.round_active {
            if let Some(start) = self.state.round_start_time {
                self.state.round_elapsed_secs = start.elapsed().as_secs_f32();
            }
            ctx.request_repaint();
        }

        // 録音中なら録音経過時間も更新
        if self.state.is_recording {
            if let Some(recorder) = &resources.voice_recorder {
                self.state.elapsed_secs = recorder.elapsed_secs();
            }
        }

        // 語彙が変わっていれば認識コンテキストを組み直す
        self.refresh_context(resources);

        // 書き起こし結果をポーリング
        self.poll_transcription_results();

        // 録音中ならVADチェックしてチャンクを送信
        if self.state.is_recording {
            self.check_vad_and_send(resources);
        }

        // ダウンロード進捗を更新
        self.update_download_progress(resources);
    }

    /// アプリ終了時の後始末（`eframe::App::on_exit` から呼ぶ）。
    /// 録音ストリームを止め、ダウンロードを中断・join し、書き起こしスレッドを終了・join する。
    pub fn shutdown(&mut self, resources: &mut Resources) {
        // 録音ストリームを停止（最終チャンクの書き起こしは行わない）
        if self.state.is_recording {
            if let Some(recorder) = &mut resources.voice_recorder {
                recorder.stop_recording();
            }
            self.state.is_recording = false;
        }

        // ダウンロード中なら中断して join（次の読み込みチャンクで抜けるためすぐ終わる）
        self.download_cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.download_handle.take() {
            let _ = handle.join();
        }

        // 書き起こしスレッドを終了・join（TranscriberThread の Drop が join する）
        self.transcriber_thread = None;
    }

    /// Action の適用（中央 dispatch の③）。
    pub fn handle_action(&mut self, resources: &mut Resources, action: VoiceMemoAction) {
        match action {
            VoiceMemoAction::StartRound => self.start_round(resources),
            VoiceMemoAction::EndRound => self.end_round(resources),
            VoiceMemoAction::StartRecording => self.start_recording(resources),
            VoiceMemoAction::StopRecording => self.stop_recording(resources),
            VoiceMemoAction::SelectRound(index) => self.select_round(index),
            VoiceMemoAction::ClearAllRounds => self.clear_all_rounds(),
            VoiceMemoAction::DownloadModel => self.start_download(resources),
        }
    }
}

impl Default for VoiceMemoFeature {
    fn default() -> Self {
        Self {
            state: VoiceMemoState::default(),
            download_rx: None,
            download_handle: None,
            download_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            context: None,
            vocabulary: None,
            context_vocabulary: None,
            transcriber_thread: None,
            speech_start_sample: 0,
            last_vad_check_sample: 0,
            silence_start: None,
            is_speaking: false,
            recording_start_round_secs: 0.0,
        }
    }
}
