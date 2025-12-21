#[cfg(not(target_arch = "wasm32"))]
mod recorder;
#[cfg(not(target_arch = "wasm32"))]
mod transcriber;
#[cfg(not(target_arch = "wasm32"))]
mod view;

#[cfg(not(target_arch = "wasm32"))]
pub use recorder::AudioRecorder;
#[cfg(not(target_arch = "wasm32"))]
pub use transcriber::{get_model_path, model_exists, WhisperTranscriber};

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
    recorder: Option<AudioRecorder>,
    transcriber: Option<WhisperTranscriber>,
    /// View用の状態（pub(crate)でViewからアクセス可能）
    pub(crate) state: VoiceMemoState,
    /// ダウンロード進捗受信用
    download_rx: Option<std::sync::mpsc::Receiver<transcriber::DownloadProgress>>,
    /// ダウンロードスレッドハンドル
    download_handle: Option<std::thread::JoinHandle<Result<(), String>>>,
    /// 認識用コンテキスト（プレイヤー名など）
    context: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl VoiceMemoFeature {
    /// AppStateを受け取って音声メモウィンドウを描画
    /// MainView::render_windows から呼び出される
    /// ゲームが開始されている場合は常に表示
    pub fn render_with_state(state: &mut crate::state::AppState, ctx: &egui::Context) {
        use crate::i18n::words::ja::JapaneseWords;
        use crate::models::Area;

        // ゲームがなければ何もしない
        if state.game().is_none() {
            return;
        }

        // 1. コンテキスト情報を収集（AppStateから）
        let player_info: Vec<(String, String)> = state
            .players()
            .map(|players| {
                players
                    .iter()
                    .map(|p| (p.name.clone(), p.color.name_ja().to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let room_names: &[&str] = match state.area() {
            Some(Area::Skeld) => JapaneseWords::SKELD_ROOMS,
            Some(Area::Mira) => JapaneseWords::MIRA_ROOMS,
            Some(Area::Polus) => JapaneseWords::POLUS_ROOMS,
            Some(Area::AirShip) | None => JapaneseWords::AIRSHIP_ROOMS,
        };

        // 2. VoiceMemoの状態更新（コンテキスト設定、タイマー更新など）
        {
            let voice_memo = state.voice_memo_mut();
            voice_memo.set_context(&player_info, room_names);
            voice_memo.update(ctx);
        }

        // 3. Viewを描画してActionsを取得
        let actions = egui::Window::new("音声メモ")
            .collapsible(true)
            .resizable(true)
            .default_size([300.0, 400.0])
            .show(ctx, |ui| view::VoiceMemoView::render(&state.voice_memo_mut().state, ui))
            .and_then(|r| r.inner)
            .unwrap_or_default();

        // 4. Actionsを処理
        for action in actions {
            state.voice_memo_mut().handle_action(action);
        }
    }

    pub fn new() -> Self {
        // レコーダーを初期化
        let recorder = match AudioRecorder::new() {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("AudioRecorder初期化エラー: {}", e);
                None
            }
        };

        // Whisperモデルが存在すればトランスクライバーを初期化
        let transcriber = if model_exists() {
            match WhisperTranscriber::new(&get_model_path()) {
                Ok(t) => Some(t),
                Err(e) => {
                    eprintln!("WhisperTranscriber初期化エラー: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let state = VoiceMemoState {
            model_available: transcriber.is_some(),
            recorder_available: recorder.is_some(),
            ..Default::default()
        };

        Self {
            recorder,
            transcriber,
            state,
            download_rx: None,
            download_handle: None,
            context: None,
        }
    }

    /// 認識用コンテキストを設定（プレイヤー名、カラー、部屋名）
    /// players: (名前, 色の日本語名) のタプル配列
    /// room_names: 現在のマップの部屋名リスト
    pub fn set_context(&mut self, players: &[(String, String)], room_names: &[&str]) {
        // プレイヤー情報を「名前(色)」形式で結合
        let player_info: Vec<String> = players
            .iter()
            .filter(|(name, _)| !name.is_empty())
            .map(|(name, color)| format!("{}({})", name, color))
            .collect();

        // 部屋名を結合（重複を除去）
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

        // eprintln!("音声認識コンテキスト: {}", context);
        self.context = Some(context);
    }

    /// 状態更新（タイマー、ダウンロード進捗など）
    /// Viewを描画する前に呼び出す
    fn update(&mut self, ctx: &egui::Context) {
        // ラウンド進行中ならタイマーを更新
        if self.state.round_active {
            if let Some(start) = self.state.round_start_time {
                self.state.round_elapsed_secs = start.elapsed().as_secs_f32();
            }
            ctx.request_repaint();
        }

        // 録音中なら録音経過時間も更新
        if self.state.is_recording {
            if let Some(recorder) = &self.recorder {
                self.state.elapsed_secs = recorder.elapsed_secs();
            }
        }

        // ダウンロード進捗を更新
        self.update_download_progress();
    }

    /// ダウンロード進捗を更新
    fn update_download_progress(&mut self) {
        if let Some(rx) = &self.download_rx {
            // 最新の進捗を取得（複数あれば最後のものを使用）
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

        // ダウンロード完了チェック
        if self.state.is_downloading {
            if let Some(handle) = self.download_handle.take() {
                if handle.is_finished() {
                    match handle.join() {
                        Ok(Ok(())) => {
                            // ダウンロード成功、トランスクライバーを初期化
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;

                            match WhisperTranscriber::new(&get_model_path()) {
                                Ok(t) => {
                                    self.transcriber = Some(t);
                                    self.state.model_available = true;
                                    self.state.error = None;
                                }
                                Err(e) => {
                                    self.state.error = Some(format!("モデル読み込み失敗: {}", e));
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
                    // まだ完了していない、ハンドルを戻す
                    self.download_handle = Some(handle);
                }
            }
        }
    }

    fn handle_action(&mut self, action: VoiceMemoAction) {
        match action {
            VoiceMemoAction::StartRound => self.start_round(),
            VoiceMemoAction::EndRound => self.end_round(),
            VoiceMemoAction::StartRecording => self.start_recording(),
            VoiceMemoAction::StopRecording => self.stop_recording(),
            VoiceMemoAction::SelectRound(index) => self.select_round(index),
            VoiceMemoAction::ClearAllRounds => self.clear_all_rounds(),
            VoiceMemoAction::DownloadModel => self.start_download(),
        }
    }

    fn start_round(&mut self) {
        // 新しいラウンドを開始
        self.state.rounds.push(Round::default());
        self.state.selected_round = self.state.rounds.len() - 1;
        self.state.round_active = true;
        self.state.round_start_time = Some(std::time::Instant::now());
        self.state.round_elapsed_secs = 0.0;

        // 録音も同時に開始
        self.do_start_recording();
    }

    fn do_start_recording(&mut self) {
        if let Some(recorder) = &mut self.recorder {
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

    fn end_round(&mut self) {
        // ラウンドの経過時間を記録（現在時刻から直接計算）
        if let Some(round) = self.state.rounds.last_mut() {
            let duration = self
                .state
                .round_start_time
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(self.state.round_elapsed_secs);
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

    fn start_download(&mut self) {
        if self.state.is_downloading {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.is_downloading = true;
        self.state.download_progress = Some(0.0);
        self.state.error = None;

        let handle = std::thread::spawn(move || transcriber::download_model(tx));
        self.download_handle = Some(handle);
    }

    fn start_recording(&mut self) {
        // ラウンドが開始されていない場合は自動で開始（録音も同時に開始される）
        if !self.state.round_active {
            self.start_round();
            return;
        }

        self.do_start_recording();
    }

    fn stop_recording(&mut self) {
        if let Some(recorder) = &mut self.recorder {
            let current_round = self.state.rounds.len().saturating_sub(1);
            // 録音開始時のラウンド経過時間（これを各発話のタイムスタンプに加算）
            let recording_start_time = self.state.round_elapsed_secs - self.state.elapsed_secs;

            match recorder.stop_recording() {
                Ok(segment) => {
                    self.state.is_recording = false;
                    self.state.is_processing = true;

                    // 書き起こし実行
                    if let Some(transcriber) = &self.transcriber {
                        match transcriber.transcribe(&segment.samples, self.context.as_deref()) {
                            Ok(memos) => {
                                // 各発話を個別のメモとして追加
                                // タイムスタンプは録音開始時刻 + Whisperが検出した相対時刻
                                if let Some(round) = self.state.rounds.get_mut(current_round) {
                                    for memo in memos {
                                        if !memo.text.trim().is_empty() {
                                            round.memos.push(VoiceMemo {
                                                timestamp_secs: recording_start_time + memo.timestamp_secs,
                                                text: memo.text.trim().to_string(),
                                            });
                                        }
                                    }
                                }
                                self.state.is_processing = false;
                                // 録音停止でラウンドも終了
                                self.end_round();
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

    /// 選択中のラウンドのメモ一覧を取得
    pub fn memos(&self) -> &[VoiceMemo] {
        self.state
            .rounds
            .get(self.state.selected_round)
            .map(|r| r.memos.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for VoiceMemoFeature {
    fn default() -> Self {
        Self::new()
    }
}
