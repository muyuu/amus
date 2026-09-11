//! ラウンドと録音のライフサイクル（開始・停止・選択・クリア）

use super::vad::MIN_SPEECH_SECS;
use super::{Round, VoiceMemoFeature};
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::Resources;
use crate::{log_debug, log_error};

impl VoiceMemoFeature {
    pub(super) fn start_round(&mut self, resources: &mut Resources) {
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
                    self.state.error = Some(e.to_string());
                }
            }
        }
    }

    pub(super) fn end_round(&mut self, resources: &mut Resources) {
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

    pub(super) fn select_round(&mut self, index: usize) {
        if index < self.state.rounds.len() {
            self.state.selected_round = index;
        }
    }

    pub(super) fn clear_all_rounds(&mut self) {
        self.state.rounds.clear();
        self.state.selected_round = 0;
        self.state.round_active = false;
        self.state.round_start_time = None;
        self.state.round_elapsed_secs = 0.0;
    }

    pub(super) fn start_recording(&mut self, resources: &mut Resources) {
        if !self.state.round_active {
            // ラウンドが開始されていない場合は何もしない
            // （UIからは呼ばれないはず）
            return;
        }

        self.do_start_recording(resources);
    }

    pub(super) fn stop_recording(&mut self, resources: &mut Resources) {
        if let Some(recorder) = &mut resources.voice_recorder {
            // 発話中なら残りのサンプルを取得（停止で取得できなくなる前に）
            let remaining = if self.is_speaking {
                Some(recorder.get_samples_since(self.speech_start_sample))
            } else {
                None
            };

            // 録音を停止
            recorder.stop_recording();
            self.state.is_recording = false;

            // 送信に必要な最小サンプル数
            let min_speech_samples = (SAMPLE_RATE as f32 * MIN_SPEECH_SECS) as usize;

            // 発話中だった場合は残りのサンプルを送信
            if let Some(slice) = remaining {
                let samples = slice.samples;
                if samples.len() >= min_speech_samples && self.transcriber_thread.is_some() {
                    let duration_secs = samples.len() as f32 / SAMPLE_RATE as f32;
                    log_debug!(
                        "VoiceMemo",
                        &format!(
                            "録音停止: 最終発話チャンク送信 ({:.1}秒), pending={}",
                            duration_secs,
                            self.state.pending_chunks + 1
                        )
                    );
                    self.send_transcription_request(samples, slice.start);
                } else {
                    let duration_secs = samples.len() as f32 / SAMPLE_RATE as f32;
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
    }
}
