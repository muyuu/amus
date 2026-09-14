//! 音声メモ機能
//!
//! 音声を録音して Whisper で書き起こす機能（ネイティブ専用）。
//!
//! 責務はファイルに分割している:
//! - `types`: DTO / View 用状態
//! - `vad`: 発話区間検出
//! - `reading`: 読みの正規化と近似マッチ（hotword と vocabulary が共有する）
//! - `hotword`: トリガーワードの照合とターン境界の決定
//! - `vocabulary`: 書き起こし結果を既知語彙へ寄せる
//! - `transcription`: 書き起こしスレッドとのやり取り・結果の取り込み
//! - `download`: Whisper モデルのダウンロード制御
//! - `recording`: ターン・録音のライフサイクル
//! - `view`: 描画

mod download;
mod hotword;
mod prompt;
mod reading;
mod recording;
mod transcription;
mod types;
mod vad;
pub mod view;
mod vocabulary;

pub use types::{Round, VoiceMemo, VoiceMemoState};

use hotword::{BoundaryTracker, HotwordMatcher};
use vocabulary::VocabularyCorrector;

use crate::app_action::AppAction;
use crate::i18n::keys as K;
use crate::i18n::words::ja::JapaneseWords;
use crate::log_debug;
use crate::log_error;
use crate::models::Area;
use crate::resources::speech::SpeechDetector;
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::{DownloadError, DownloadProgress, Resources, TranscriberThread};
use crate::state::Slices;

/// 音声メモのアクション
#[derive(Debug, Clone)]
pub enum VoiceMemoAction {
    StartRound,
    EndRound,
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
    /// ゲームが開始されているか。描画時に Slices から拾う。
    game_active: bool,
    /// 描画時に拾った最新のゲーム世代
    game_generation: u64,
    /// 追従済みのゲーム世代。これと違えばゲームが作り直されている。
    observed_game_generation: u64,
    /// 描画時に集めた最新の語彙
    vocabulary: Option<prompt::Vocabulary>,
    /// `context` を組み立てた元の語彙。変化したときだけ組み直す。
    context_vocabulary: Option<prompt::Vocabulary>,
    /// 書き起こし結果を既知語彙へ寄せる補正器
    corrector: VocabularyCorrector,
    /// バックグラウンド書き起こしスレッド
    transcriber_thread: Option<TranscriberThread>,
    /// 発話区間の検出器
    detector: SpeechDetector,
    /// VAD で検査済みの録音上の位置
    vad_checked_sample: usize,
    /// 現在の録音が始まった絶対サンプル位置
    recording_start_sample: usize,

    /// トリガーワードの照合器
    matcher: HotwordMatcher,
    /// 検出をターン境界へ変換する
    boundary_tracker: BoundaryTracker,
}

impl VoiceMemoFeature {
    pub fn new(resources: &Resources) -> Self {
        let model_available = resources.whisper_transcriber.is_some();
        let state = VoiceMemoState {
            model_available,
            recorder_available: resources.voice_recorder.is_some(),
            ..Default::default()
        };

        let mut feature = Self {
            state,
            ..Default::default()
        };
        feature.sync_backend_state(resources);
        if model_available {
            feature.restart_transcriber_thread(resources);
        }
        feature
    }

    /// 構成まわりの表示を Resources の実態に合わせる。
    fn sync_backend_state(&mut self, resources: &Resources) {
        self.state.fallback_label = resources.transcribe_fallback().map(|s| s.label());
        self.state.model_size_label = resources.transcribe_setup().model.size_label();
    }

    /// 書き起こしスレッドを今の構成で起動し直す。
    ///
    /// 前のスレッドは drop で join されるため、処理中のチャンクの結果は返ってこない。
    /// 数え続けると「書き起こし中」が消えなくなるため、待ち数もここで捨てる。
    fn restart_transcriber_thread(&mut self, resources: &Resources) {
        self.transcriber_thread = None;
        self.state.pending_chunks = 0;
        self.state.is_processing = false;

        match TranscriberThread::new(resources.transcribe_setup()) {
            Ok(t) => self.transcriber_thread = Some(t),
            Err(e) => {
                log_error!(
                    "VoiceMemo",
                    &format!("TranscriberThread初期化エラー: {}", e)
                );
            }
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

        // 2. ゲームの有無と認識語彙を記録（録音・プロンプトの反映は update 側）
        self.game_active = slices.game().has_game();
        self.game_generation = slices.game().generation();
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

    /// ゲームの有無と作り直しに録音を追従させる。
    ///
    /// 録音はターンの区間を切り出すための土台であり、ユーザーが意識する操作ではない。
    /// ゲームが始まっている間は回し続け、終われば止める。
    ///
    /// ゲームが作り直されたら前のゲームのメモと録音は破棄する。ターンは録音上の位置で
    /// 区間を持つため、録音を取り直しつつメモを残すと位置の対応が崩れる。
    fn follow_game_lifecycle(&mut self, resources: &mut Resources) {
        if self.game_generation != self.observed_game_generation {
            self.observed_game_generation = self.game_generation;
            self.stop_recording(resources);
            self.clear_all_rounds();
        }

        match (self.game_active, self.state.is_recording) {
            (true, false) => self.start_recording(resources),
            (false, true) => self.stop_recording(resources),
            _ => {}
        }
    }

    /// マイクデバイスを切り替える前に呼ぶ。
    ///
    /// 録音中なら（溜まっている発話の書き起こし送信・進行中ターンのクローズを含めて）
    /// 安全に止める。録音していなければ何もしない。ゲームが進行中なら、次フレームの
    /// `follow_game_lifecycle` が新しい録音リソースで自動的に録音を再開する。
    pub fn stop_recording_for_hardware_change(&mut self, resources: &mut Resources) {
        if self.state.is_recording {
            self.stop_recording(resources);
        }
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

        let context =
            prompt::build_recognition_context(vocabulary, |text| transcriber.count_tokens(text));

        // 認識結果が期待と違うとき、語彙がプロンプトに載っていないのか、載った上で
        // モデルが採用しなかったのかを切り分けるために残す。発話内容ではなく設定値。
        log_debug!(
            "VoiceMemo",
            format!(
                "認識コンテキストを更新: {}トークン / {}",
                transcriber.count_tokens(&context),
                context
            )
        );

        self.context = Some(context);
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
        let vocabulary = prompt::Vocabulary {
            player_names: players
                .iter()
                .map(|(name, _)| name.clone())
                .filter(|name| !name.is_empty())
                .collect(),
            room_names,
        };

        if self.vocabulary.as_ref() == Some(&vocabulary) {
            return;
        }

        self.corrector = VocabularyCorrector::new(vocabulary.known_terms());
        self.vocabulary = Some(vocabulary);
    }

    /// リアルタイム更新（中央 dispatch の①）。録音の維持・タイマー・ポーリング・VAD・
    /// ダウンロード進捗。
    pub fn update(&mut self, resources: &mut Resources, ctx: &egui::Context) {
        self.follow_game_lifecycle(resources);

        // 録音中なら経過時間を更新。ターンの経過は録音位置から求める。
        if self.state.is_recording {
            if let Some(recorder) = &resources.voice_recorder {
                if self.state.round_active {
                    let round_start = self
                        .state
                        .rounds
                        .last()
                        .map(|round| round.start_sample)
                        .unwrap_or(0);
                    self.state.round_elapsed_secs =
                        recorder.buffer_len().saturating_sub(round_start) as f32
                            / crate::resources::voice_recorder::SAMPLE_RATE as f32;
                }
            }
            ctx.request_repaint();
        }

        // 語彙が変わっていれば認識コンテキストを組み直す
        self.refresh_context(resources);

        // 書き起こし結果をポーリング。ターン境界もここで決まる。
        self.poll_transcription_results();

        // 録音中はターン外もVADチェックしてチャンクを送信。ターンはトリガーワードの
        // 書き起こしで開くため、開く前の発話も書き起こしておく必要がある。
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
            game_active: false,
            game_generation: 0,
            observed_game_generation: 0,
            vocabulary: None,
            context_vocabulary: None,
            corrector: VocabularyCorrector::new([]),
            transcriber_thread: None,
            detector: SpeechDetector::default(),
            vad_checked_sample: 0,
            recording_start_sample: 0,
            matcher: HotwordMatcher::new(hotword::DEFAULT_START_WORDS, hotword::DEFAULT_END_WORDS),
            boundary_tracker: BoundaryTracker::new(
                (SAMPLE_RATE as f32 * hotword::COOLDOWN_SECS) as usize,
            ),
        }
    }
}
