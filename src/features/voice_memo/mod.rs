//! 音声メモ機能
//!
//! 音声を録音して Whisper で書き起こす機能。

#[cfg(not(target_arch = "wasm32"))]
pub mod view;

use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::i18n::keys as K;
#[cfg(not(target_arch = "wasm32"))]
use crate::i18n::words::ja::JapaneseWords;
#[cfg(not(target_arch = "wasm32"))]
use crate::log_debug;
#[cfg(not(target_arch = "wasm32"))]
use crate::log_error;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::Area;
#[cfg(not(target_arch = "wasm32"))]
use crate::resources::{
    download_file, get_model_download_url, get_model_path, DownloadProgress, Resources,
    TranscribeRequest, TranscriberThread,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::state::AppState;

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

/// 音声メモのアクション
#[cfg(not(target_arch = "wasm32"))]
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

/// VAD（発話区間検出）の設定
#[cfg(not(target_arch = "wasm32"))]
const SILENCE_THRESHOLD: f32 = 0.01; // 無音判定の閾値（RMS）
#[cfg(not(target_arch = "wasm32"))]
const SILENCE_DURATION_SECS: f32 = 1.5; // 無音がこの秒数続いたら発話終了とみなす
#[cfg(not(target_arch = "wasm32"))]
const MIN_SPEECH_SECS: f32 = 1.0; // 最低1秒分の発話がないと送信しない
#[cfg(not(target_arch = "wasm32"))]
const VAD_WINDOW_SECS: f32 = 0.1; // 100msのウィンドウでRMSを計算

/// 音声メモ機能
#[cfg(not(target_arch = "wasm32"))]
pub struct VoiceMemoFeature {
    /// View用の状態
    pub state: VoiceMemoState,
    /// ダウンロード進捗受信用
    download_rx: Option<std::sync::mpsc::Receiver<DownloadProgress>>,
    /// ダウンロードスレッドハンドル
    download_handle: Option<std::thread::JoinHandle<Result<(), String>>>,
    /// 認識用コンテキスト（プレイヤー名など）
    context: Option<String>,
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

#[cfg(not(target_arch = "wasm32"))]
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
            context: None,
            transcriber_thread,
            speech_start_sample: 0,
            last_vad_check_sample: 0,
            silence_start: None,
            is_speaking: false,
            recording_start_round_secs: 0.0,
        }
    }

    /// 音声メモウィンドウを描画
    pub fn render(&mut self, app_state: &AppState, resources: &mut Resources, ui: &mut egui::Ui) {
        // 1. コンテキスト情報を収集（AppStateから）
        let player_info: Vec<(String, String)> = app_state
            .players()
            .map(|players| {
                players
                    .iter()
                    .map(|p| (p.name.clone(), p.color.name_ja().to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let room_names: &[&str] = match app_state.area() {
            Some(Area::Skeld) => JapaneseWords::SKELD_ROOMS,
            Some(Area::Mira) => JapaneseWords::MIRA_ROOMS,
            Some(Area::Polus) => JapaneseWords::POLUS_ROOMS,
            Some(Area::AirShip) | None => JapaneseWords::AIRSHIP_ROOMS,
        };

        // 2. コンテキスト設定
        self.set_context(&player_info, room_names);

        // 3. 状態更新
        self.update(resources, ui.ctx());

        // 4. Viewを描画してActionsを取得
        let translator = app_state.translator();
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

        // 5. Actionsを処理
        for action in actions {
            self.handle_action(resources, action);
        }
    }

    fn set_context(&mut self, players: &[(String, String)], room_names: &[&str]) {
        let player_info: Vec<String> = players
            .iter()
            .filter(|(name, _)| !name.is_empty())
            .map(|(name, color)| format!("{}({})", name, color))
            .collect();

        let rooms: Vec<&str> = room_names.to_vec();
        let rooms_str = rooms.join("、");

        // i18n/words/ja.rs のアモアス用語を使用
        let amongus_terms = [
            JapaneseWords::SABOTAGE,
            JapaneseWords::SABOTAGE_COMMS,
            JapaneseWords::SABOTAGE_LIGHTS,
            JapaneseWords::SABOTAGE_O2,
            JapaneseWords::SABOTAGE_REACTOR,
            JapaneseWords::SABOTAGE_DOORS,
            JapaneseWords::SPAWN,
            JapaneseWords::VENT,
            JapaneseWords::TASK,
            JapaneseWords::DISCUSSION,
        ];

        let terms_str = amongus_terms.join("、");

        let context = if player_info.is_empty() {
            format!(
                "Among Usのゲーム。部屋: {}。用語: {}。",
                rooms_str, terms_str
            )
        } else {
            format!(
                "Among Usのゲーム。プレイヤー: {}。部屋: {}。用語: {}。",
                player_info.join("、"),
                rooms_str,
                terms_str
            )
        };

        self.context = Some(context);
    }

    fn update(&mut self, resources: &Resources, ctx: &egui::Context) {
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

        // 書き起こし結果をポーリング
        self.poll_transcription_results();

        // 録音中ならVADチェックしてチャンクを送信
        if self.state.is_recording {
            self.check_vad_and_send(resources);
        }

        // ダウンロード進捗を更新
        self.update_download_progress(resources);
    }

    /// 書き起こし結果をポーリング
    fn poll_transcription_results(&mut self) {
        if let Some(transcriber) = &self.transcriber_thread {
            let results = transcriber.poll_results();
            for result in results {
                self.state.pending_chunks = self.state.pending_chunks.saturating_sub(1);

                if let Some(error) = result.error {
                    log_error!("VoiceMemo", &format!("書き起こしエラー: {}", error));
                    self.state.error = Some(error);
                    continue;
                }

                // 結果をラウンドに追加
                if let Some(round) = self.state.rounds.get_mut(result.round_index) {
                    let mut added_count = 0;
                    for seg in result.segments {
                        if !seg.text.trim().is_empty() {
                            round.memos.push(VoiceMemo {
                                timestamp_secs: seg.timestamp_secs,
                                text: seg.text.trim().to_string(),
                            });
                            added_count += 1;
                        }
                    }
                    // タイムスタンプでソート
                    round.memos.sort_by(|a, b| {
                        a.timestamp_secs
                            .partial_cmp(&b.timestamp_secs)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    log_debug!(
                        "VoiceMemo",
                        &format!("解析完了: {}件のメモを追加", added_count)
                    );
                }
            }

            // 処理待ちがなくなったらis_processingをfalseに
            if self.state.pending_chunks == 0 {
                self.state.is_processing = false;
            }
        }
    }

    /// VADチェックして発話終了時にチャンクを送信
    fn check_vad_and_send(&mut self, resources: &Resources) {
        let Some(recorder) = &resources.voice_recorder else {
            return;
        };

        let sample_rate = recorder.sample_rate();
        let vad_window_samples = (sample_rate as f32 * VAD_WINDOW_SECS) as usize;

        let buffer_len = recorder.buffer_len();
        if buffer_len <= self.last_vad_check_sample + vad_window_samples {
            return; // 新しいサンプルが足りない
        }

        // 最新のウィンドウでRMSを計算（元サンプルレートのまま）
        let window_start = buffer_len.saturating_sub(vad_window_samples);
        let (recent_samples, _) = recorder.get_samples_since(window_start);
        let rms = Self::calculate_rms(&recent_samples);
        let is_sound = rms > SILENCE_THRESHOLD;

        self.last_vad_check_sample = buffer_len;

        if is_sound {
            // 音声あり
            if !self.is_speaking {
                // 発話開始
                self.is_speaking = true;
                self.speech_start_sample = window_start;
                log_debug!(
                    "VoiceMemo",
                    &format!("発話開始検出 (RMS={:.4}, sample_rate={})", rms, sample_rate)
                );
            }
            self.silence_start = None;
        } else {
            // 無音
            if self.is_speaking {
                // 発話中に無音を検出
                let silence_start = self
                    .silence_start
                    .get_or_insert_with(std::time::Instant::now);
                let silence_duration = silence_start.elapsed().as_secs_f32();

                if silence_duration >= SILENCE_DURATION_SECS {
                    // 無音が十分続いた → 発話終了、チャンクを送信
                    log_debug!(
                        "VoiceMemo",
                        &format!("無音検知、解析開始 (無音継続={:.1}s)", silence_duration)
                    );
                    self.send_speech_chunk(resources, sample_rate);
                }
            }
        }
    }

    /// RMS（二乗平均平方根）を計算
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    /// 発話チャンクを送信
    fn send_speech_chunk(&mut self, resources: &Resources, sample_rate: u32) {
        let Some(recorder) = &resources.voice_recorder else {
            return;
        };

        // 発話区間のサンプルを取得（16kHzにリサンプリング済み）
        let (samples, next_pos) = recorder.get_samples_since(self.speech_start_sample);

        // 16kHzでの最小サンプル数
        let min_speech_samples = (16000.0 * MIN_SPEECH_SECS) as usize;

        if samples.len() < min_speech_samples {
            let duration_secs = samples.len() as f32 / 16000.0;
            log_debug!(
                "VoiceMemo",
                &format!(
                    "発話が短すぎるためスキップ ({:.1}秒, {}サンプル@16kHz)",
                    duration_secs,
                    samples.len()
                )
            );
            self.reset_vad_state();
            return;
        }

        let duration_secs = samples.len() as f32 / 16000.0;
        log_debug!(
            "VoiceMemo",
            &format!(
                "発話チャンク準備: {:.1}秒分 ({}サンプル@16kHz, 元rate={})",
                duration_secs,
                samples.len(),
                sample_rate
            )
        );

        self.send_transcription_request(samples, next_pos);
        self.reset_vad_state();
    }

    /// VAD状態をリセット
    fn reset_vad_state(&mut self) {
        self.is_speaking = false;
        self.silence_start = None;
        self.speech_start_sample = self.last_vad_check_sample;
    }

    /// 書き起こしリクエストを送信
    fn send_transcription_request(&mut self, samples: Vec<f32>, _next_sample_pos: usize) {
        if let Some(transcriber) = &self.transcriber_thread {
            let current_round = self.state.rounds.len().saturating_sub(1);

            // オフセット計算: 録音開始時のラウンド経過時間 + 発話開始位置
            let chunk_start_secs =
                self.recording_start_round_secs + (self.speech_start_sample as f32 / 48000.0); // 元サンプルレート（概算）

            let request = TranscribeRequest {
                samples,
                offset_secs: chunk_start_secs,
                context: self.context.clone(),
                round_index: current_round,
            };

            transcriber.request(request);
            self.state.pending_chunks += 1;
            self.state.is_processing = true;

            log_debug!(
                "VoiceMemo",
                &format!(
                    "発話チャンク送信: offset={:.1}s, round={}",
                    chunk_start_secs, current_round
                )
            );
        }
    }

    fn update_download_progress(&mut self, _resources: &Resources) {
        if let Some(rx) = &self.download_rx {
            let mut latest_progress = None;
            while let Ok(progress) = rx.try_recv() {
                latest_progress = Some(progress);
            }

            if let Some(progress) = latest_progress {
                self.state.download_progress = progress.percentage();
                self.state.downloaded_bytes = progress.downloaded_bytes;
                self.state.total_bytes = progress.total_bytes;
            }
        }

        if self.state.is_downloading {
            if let Some(handle) = self.download_handle.take() {
                if handle.is_finished() {
                    match handle.join() {
                        Ok(Ok(())) => {
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;
                            self.state.model_available = true;
                            self.state.error = None;
                            // TranscriberThread を初期化
                            if self.transcriber_thread.is_none() {
                                match TranscriberThread::new() {
                                    Ok(t) => {
                                        self.transcriber_thread = Some(t);
                                        log_debug!("VoiceMemo", "TranscriberThread初期化完了");
                                    }
                                    Err(e) => {
                                        log_error!(
                                            "VoiceMemo",
                                            &format!("TranscriberThread初期化エラー: {}", e)
                                        );
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;
                            self.state.error = Some(e);
                        }
                        Err(_) => {
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;
                            self.state.error = Some("ダウンロードスレッドがパニック".to_string());
                        }
                    }
                } else {
                    self.download_handle = Some(handle);
                }
            }
        }
    }

    fn handle_action(&mut self, resources: &mut Resources, action: VoiceMemoAction) {
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

    fn start_round(&mut self, resources: &mut Resources) {
        self.state.rounds.push(Round::default());
        self.state.selected_round = self.state.rounds.len() - 1;
        self.state.round_active = true;
        self.state.round_start_time = Some(std::time::Instant::now());
        self.state.round_elapsed_secs = 0.0;

        self.do_start_recording(resources);
    }

    fn do_start_recording(&mut self, resources: &mut Resources) {
        if let Some(recorder) = &mut resources.voice_recorder {
            match recorder.start_recording() {
                Ok(_) => {
                    self.state.is_recording = true;
                    self.state.error = None;
                    // VAD状態をリセット
                    self.speech_start_sample = 0;
                    self.last_vad_check_sample = 0;
                    self.silence_start = None;
                    self.is_speaking = false;
                    self.recording_start_round_secs = self.state.round_elapsed_secs;
                    log_debug!(
                        "VoiceMemo",
                        &format!(
                            "録音開始: round_elapsed={:.1}s",
                            self.recording_start_round_secs
                        )
                    );
                }
                Err(e) => {
                    log_error!("VoiceMemo", &format!("録音開始エラー: {}", e));
                    self.state.error = Some(e);
                }
            }
        }
    }

    fn end_round(&mut self, resources: &mut Resources) {
        // 先にラウンド時間を確定（書き起こし時間を含めないため）
        let duration = self
            .state
            .round_start_time
            .map(|start| start.elapsed().as_secs_f32())
            .unwrap_or(self.state.round_elapsed_secs);

        // 録音中なら停止（書き起こしも実行）
        if self.state.is_recording {
            self.stop_recording(resources);
        }

        if let Some(round) = self.state.rounds.last_mut() {
            round.duration_secs = Some(duration);
        }
        self.state.round_active = false;
        self.state.round_start_time = None;
    }

    fn select_round(&mut self, index: usize) {
        if index < self.state.rounds.len() {
            self.state.selected_round = index;
        }
    }

    fn clear_all_rounds(&mut self) {
        self.state.rounds.clear();
        self.state.selected_round = 0;
        self.state.round_active = false;
        self.state.round_start_time = None;
        self.state.round_elapsed_secs = 0.0;
    }

    fn start_download(&mut self, resources: &mut Resources) {
        if self.state.is_downloading {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.is_downloading = true;
        self.state.download_progress = Some(0.0);
        self.state.error = None;

        let url = get_model_download_url().to_string();
        let dest_path = get_model_path();

        let handle = std::thread::spawn(move || download_file(&url, &dest_path, tx));
        self.download_handle = Some(handle);

        // ダウンロード完了後に Resources の whisper_transcriber を初期化する必要がある
        // これは update_download_progress で処理される
        let _ = resources; // 将来の拡張用
    }

    fn start_recording(&mut self, resources: &mut Resources) {
        if !self.state.round_active {
            // ラウンドが開始されていない場合は何もしない
            // （UIからは呼ばれないはず）
            return;
        }

        self.do_start_recording(resources);
    }

    fn stop_recording(&mut self, resources: &mut Resources) {
        if let Some(recorder) = &mut resources.voice_recorder {
            // 発話中なら残りのサンプルを取得
            let remaining = if self.is_speaking {
                let (samples, next_pos) = recorder.get_samples_since(self.speech_start_sample);
                Some((samples, next_pos))
            } else {
                None
            };

            // 録音を停止
            match recorder.stop_recording() {
                Ok(()) => {
                    self.state.is_recording = false;

                    // 16kHzでの最小サンプル数
                    let min_speech_samples = (16000.0 * MIN_SPEECH_SECS) as usize;

                    // 発話中だった場合は残りのサンプルを送信
                    if let Some((samples, next_pos)) = remaining {
                        if samples.len() >= min_speech_samples && self.transcriber_thread.is_some()
                        {
                            let duration_secs = samples.len() as f32 / 16000.0;
                            log_debug!(
                                "VoiceMemo",
                                &format!(
                                    "録音停止: 最終発話チャンク送信 ({:.1}秒), pending={}",
                                    duration_secs,
                                    self.state.pending_chunks + 1
                                )
                            );
                            self.send_transcription_request(samples, next_pos);
                        } else {
                            let duration_secs = samples.len() as f32 / 16000.0;
                            log_debug!(
                                "VoiceMemo",
                                &format!(
                                    "録音停止: 発話が短すぎるためスキップ ({:.1}秒)",
                                    duration_secs
                                )
                            );
                        }
                    } else {
                        log_debug!("VoiceMemo", "録音停止: 発話中ではなかった");
                    }

                    // VAD状態をリセット
                    self.reset_vad_state();
                }
                Err(e) => {
                    log_error!("VoiceMemo", &format!("録音停止エラー: {}", e));
                    self.state.is_recording = false;
                    self.state.error = Some(e);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for VoiceMemoFeature {
    fn default() -> Self {
        Self {
            state: VoiceMemoState::default(),
            download_rx: None,
            download_handle: None,
            context: None,
            transcriber_thread: None,
            speech_start_sample: 0,
            last_vad_check_sample: 0,
            silence_start: None,
            is_speaking: false,
            recording_start_round_secs: 0.0,
        }
    }
}
