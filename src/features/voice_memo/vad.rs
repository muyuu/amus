//! 録音を監視して発話区間を切り出し、書き起こしへ送る。
//!
//! どこを発話とみなすかの判定は `speech` が持つ。ここはレコーダーから音量を読み、
//! 確定した区間のサンプルを渡すところまでを受け持つ。

use super::VoiceMemoFeature;
use crate::log_debug;
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::Resources;

/// 音量を測る窓の長さ。
const VAD_WINDOW_SECS: f32 = 0.1;
/// これより短い区間は書き起こしに送らない。
///
/// 区間は前後の余韻を含むため、発話の実体はこれよりさらに短い。物音ひとつで
/// 書き起こしを回さないための下限。
pub(super) const MIN_SPEECH_SECS: f32 = 0.6;

impl VoiceMemoFeature {
    /// 新しく録音された分を検査し、発話区間が確定したら書き起こしへ送る。
    pub(super) fn check_vad_and_send(&mut self, resources: &Resources) {
        let Some(recorder) = &resources.voice_recorder else {
            return;
        };

        let window_samples = (SAMPLE_RATE as f32 * VAD_WINDOW_SECS) as usize;
        let buffer_len = recorder.buffer_len();

        // 溜まった分を窓単位で漏れなく見る。フレームが飛んでも録音上の位置は飛ばない。
        while self.vad_checked_sample + window_samples <= buffer_len {
            let window = self.vad_checked_sample..self.vad_checked_sample + window_samples;
            self.vad_checked_sample = window.end;

            let samples = recorder.get_samples_since(window.start).samples;
            let rms = Self::calculate_rms(&samples[..samples.len().min(window_samples)]);

            if let Some(speech) = self.detector.observe(window, rms) {
                self.send_speech(resources, speech);
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

    /// 確定した発話区間を書き起こしへ送る。
    fn send_speech(&mut self, resources: &Resources, speech: std::ops::Range<usize>) {
        let Some(recorder) = &resources.voice_recorder else {
            return;
        };

        let slice = recorder.get_samples_since(speech.start);
        let length = (speech.end - speech.start).min(slice.samples.len());
        let samples = slice.samples[..length].to_vec();

        let duration_secs = samples.len() as f32 / SAMPLE_RATE as f32;
        if duration_secs < MIN_SPEECH_SECS {
            log_debug!(
                "VoiceMemo",
                &format!("発話が短すぎるためスキップ ({:.1}秒)", duration_secs)
            );
            return;
        }

        log_debug!(
            "VoiceMemo",
            &format!(
                "発話チャンク準備: {:.1}秒分 ({}サンプル)",
                duration_secs,
                samples.len()
            )
        );

        self.send_transcription_request(samples, slice.start);
    }

    /// 録音を取り直したときに、検出の状態を録音上の位置ごと捨てる。
    pub(super) fn reset_vad_state(&mut self, at: usize) {
        self.vad_checked_sample = at;
        self.detector.reset();
    }
}
