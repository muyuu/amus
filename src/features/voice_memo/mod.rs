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
use crate::resources::{DownloadProgress, Resources, TranscriberThread};
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

        // 2. コンテキスト設定
        self.set_context(&player_info, room_names);

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

        // 書き起こし結果をポーリング
        self.poll_transcription_results();

        // 録音中ならVADチェックしてチャンクを送信
        if self.state.is_recording {
            self.check_vad_and_send(resources);
        }

        // ダウンロード進捗を更新
        self.update_download_progress(resources);
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
