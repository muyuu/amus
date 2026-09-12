//! ターンと録音のライフサイクル（開始・停止・選択・クリア）

use super::vad::MIN_SPEECH_SECS;
use super::{Round, VoiceMemoFeature};
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::Resources;
use crate::{log_debug, log_error};

impl VoiceMemoFeature {
    /// ターンを開始する。録音中でなければ何もしない。
    ///
    /// ターンは録音上の区間なので、録音が回っていないと始点を決められない。
    pub(super) fn start_round(&mut self, resources: &mut Resources) {
        let Some(at) = self.recorded_position(resources) else {
            return;
        };
        self.open_turn(at);
    }

    /// ターンを終了して区間を確定する。録音は止めない。
    pub(super) fn end_round(&mut self, resources: &mut Resources) {
        let Some(at) = self.recorded_position(resources) else {
            return;
        };
        self.close_turn(at);
    }

    /// 指定位置でターンを開く。
    pub(super) fn open_turn(&mut self, at: usize) {
        self.state.rounds.push(Round {
            start_sample: at,
            ..Default::default()
        });
        self.state.selected_round = self.state.rounds.len() - 1;
        self.state.round_active = true;
        self.state.round_elapsed_secs = 0.0;

        log_debug!("VoiceMemo", &format!("ターン開始: start_sample={}", at));
    }

    /// 指定位置で進行中のターンを閉じる。
    pub(super) fn close_turn(&mut self, at: usize) {
        if let Some(round) = self.state.rounds.last_mut() {
            round.end_sample = Some(at);
        }
        self.state.round_active = false;

        log_debug!("VoiceMemo", &format!("ターン終了: end_sample={}", at));
    }

    /// 現在の録音位置（絶対サンプルインデックス）。録音中でなければ `None`。
    pub(super) fn recorded_position(&self, resources: &Resources) -> Option<usize> {
        if !self.state.is_recording {
            return None;
        }
        resources.voice_recorder.as_ref().map(|r| r.buffer_len())
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
        self.state.round_elapsed_secs = 0.0;
    }

    /// 録音を開始する。ターンとは独立で、ゲーム中は回しっぱなしにする。
    pub(super) fn start_recording(&mut self, resources: &mut Resources) {
        if let Some(recorder) = &mut resources.voice_recorder {
            match recorder.start_recording() {
                Ok(_) => {
                    let at = recorder.buffer_len();
                    self.state.is_recording = true;
                    self.state.error = None;
                    self.recording_start_sample = at;
                    self.reset_vad_state(at);
                    log_debug!("VoiceMemo", &format!("録音開始: at={}", at));
                }
                Err(e) => {
                    log_error!("VoiceMemo", &format!("録音開始エラー: {}", e));
                    self.state.error = Some(e.to_string());
                }
            }
        }
    }

    /// 録音を停止する。進行中のターンがあればその位置で閉じる。
    pub(super) fn stop_recording(&mut self, resources: &mut Resources) {
        // 言い終わる前に録音が止まることがある。途中まででも書き起こしに回す。
        // 送信はターンを閉じる前に行う必要がある（閉じた後は書き起こしに送られない）。
        let remaining = self.detector.pending_speech().and_then(|start| {
            resources
                .voice_recorder
                .as_ref()
                .map(|recorder| recorder.get_samples_since(start))
        });

        let min_speech_samples = (SAMPLE_RATE as f32 * MIN_SPEECH_SECS) as usize;

        if let Some(slice) = remaining {
            let samples = slice.samples;
            let duration_secs = samples.len() as f32 / SAMPLE_RATE as f32;
            if samples.len() >= min_speech_samples && self.transcriber_thread.is_some() {
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

        // 位置の取得に録音中であることが要るので、停止より先にターンを閉じる。
        if self.state.round_active {
            self.end_round(resources);
        }

        if let Some(recorder) = &mut resources.voice_recorder {
            recorder.stop_recording();
        }
        self.state.is_recording = false;
        self.reset_vad_state(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::voice_memo::VoiceMemoAction;

    /// ハードウェアを持たない Resources。録音できない状況を再現する。
    fn resources_without_hardware() -> Resources {
        Resources::without_hardware()
    }

    #[test]
    fn recreating_the_game_clears_previous_memos() {
        let mut feature = VoiceMemoFeature::default();
        let mut resources = resources_without_hardware();
        feature.state.rounds.push(Round::default());
        feature.state.round_active = true;

        // ゲームが作り直された
        feature.game_generation += 1;
        feature.follow_game_lifecycle(&mut resources);

        assert!(
            feature.state.rounds.is_empty(),
            "前のゲームのメモが残っている"
        );
        assert!(!feature.state.round_active);
    }

    #[test]
    fn starting_a_turn_without_recording_does_nothing() {
        let mut feature = VoiceMemoFeature::default();
        let mut resources = resources_without_hardware();

        feature.handle_action(&mut resources, VoiceMemoAction::StartRound);

        assert!(feature.state.rounds.is_empty(), "ターンが作られてしまった");
        assert!(!feature.state.round_active);
    }
}
