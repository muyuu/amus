//! 音声メモ機能
//!
//! 音声を録音して Whisper で書き起こす機能。

#[cfg(not(target_arch = "wasm32"))]
pub mod view;

use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::i18n::words::ja::JapaneseWords;
#[cfg(not(target_arch = "wasm32"))]
use crate::i18n::keys as K;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::Area;
#[cfg(not(target_arch = "wasm32"))]
use crate::resources::{
    download_file, get_model_download_url, get_model_path, DownloadProgress, Resources,
    WhisperTranscriber,
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
}

#[cfg(not(target_arch = "wasm32"))]
impl VoiceMemoFeature {
    pub fn new(resources: &Resources) -> Self {
        let state = VoiceMemoState {
            model_available: resources.whisper_transcriber.is_some(),
            recorder_available: resources.voice_recorder.is_some(),
            ..Default::default()
        };

        Self {
            state,
            download_rx: None,
            download_handle: None,
            context: None,
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

        let rooms: Vec<&str> = room_names.iter().copied().collect();
        let rooms_str = rooms.join("、");

        let context = if player_info.is_empty() {
            format!(
                "Among Usのゲーム実況。部屋: {}。キル、ベント、サボタージュ、タスク、会議、追放、インポスター、クルーメイト。",
                rooms_str
            )
        } else {
            format!(
                "Among Usのゲーム実況。プレイヤー: {}。部屋: {}。キル、ベント、サボタージュ、タスク、会議、追放。",
                player_info.join("、"),
                rooms_str
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

        // ダウンロード進捗を更新
        self.update_download_progress(resources);
    }

    fn update_download_progress(&mut self, resources: &Resources) {
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
                            // Note: whisper_transcriber の初期化は Resources 側で行う必要がある
                            // ここでは model_available フラグを立てるだけ
                            self.state.model_available = true;
                            self.state.error = None;
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
                }
                Err(e) => {
                    eprintln!("録音開始エラー: {}", e);
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
        let current_round = self.state.rounds.len().saturating_sub(1);
        let recording_start_time = self.state.round_elapsed_secs - self.state.elapsed_secs;

        if let Some(recorder) = &mut resources.voice_recorder {
            match recorder.stop_recording() {
                Ok(segment) => {
                    self.state.is_recording = false;
                    self.state.is_processing = true;

                    if let Some(transcriber) = &resources.whisper_transcriber {
                        match transcriber.transcribe(&segment.samples, self.context.as_deref()) {
                            Ok(segments) => {
                                if let Some(round) = self.state.rounds.get_mut(current_round) {
                                    for seg in segments {
                                        if !seg.text.trim().is_empty() {
                                            round.memos.push(VoiceMemo {
                                                timestamp_secs: recording_start_time
                                                    + seg.timestamp_secs,
                                                text: seg.text.trim().to_string(),
                                            });
                                        }
                                    }
                                }
                                self.state.is_processing = false;
                                // ラウンドは終了しない（ユーザーが明示的に終了ボタンを押すまで継続）
                            }
                            Err(e) => {
                                eprintln!("書き起こしエラー: {}", e);
                                self.state.error = Some(e);
                                self.state.is_processing = false;
                            }
                        }
                    } else {
                        self.state.error = Some("Whisperモデルが読み込まれていません".to_string());
                        self.state.is_processing = false;
                    }
                }
                Err(e) => {
                    eprintln!("録音停止エラー: {}", e);
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
        }
    }
}
