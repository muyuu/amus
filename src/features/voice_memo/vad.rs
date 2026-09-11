//! VAD（発話区間検出）。録音中のサンプルを監視し、発話の終了を検出して
//! 書き起こしチャンクを送信する。

use super::VoiceMemoFeature;
use crate::log_debug;
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::Resources;

const SILENCE_THRESHOLD: f32 = 0.01; // 無音判定の閾値（RMS）
const SILENCE_DURATION_SECS: f32 = 1.5; // 無音がこの秒数続いたら発話終了とみなす
/// 最低この秒数分の発話がないと書き起こしに送らない
pub(super) const MIN_SPEECH_SECS: f32 = 1.0;
const VAD_WINDOW_SECS: f32 = 0.1; // 100msのウィンドウでRMSを計算

impl VoiceMemoFeature {
    /// VADチェックして発話終了時にチャンクを送信
    pub(super) fn check_vad_and_send(&mut self, resources: &Resources) {
        let Some(recorder) = &resources.voice_recorder else {
            return;
        };

        let vad_window_samples = (SAMPLE_RATE as f32 * VAD_WINDOW_SECS) as usize;

        let buffer_len = recorder.buffer_len();
        if buffer_len <= self.last_vad_check_sample + vad_window_samples {
            return; // 新しいサンプルが足りない
        }

        // 最新のウィンドウでRMSを計算
        let window_start = buffer_len.saturating_sub(vad_window_samples);

        // 今後 get_samples_since で要求しうる最古の位置（発話開始 or 直近ウィンドウ）
        // より前の確定領域を破棄し、バッファの単調増加を抑える。
        let keep_from = self.speech_start_sample.min(window_start);
        recorder.discard_before(keep_from);

        let recent_samples = recorder.get_samples_since(window_start).samples;
        let rms = Self::calculate_rms(&recent_samples);
        let is_sound = rms > SILENCE_THRESHOLD;

        self.last_vad_check_sample = buffer_len;

        if is_sound {
            // 音声あり
            if !self.is_speaking {
                // 発話開始
                self.is_speaking = true;
                self.speech_start_sample = window_start;
                log_debug!("VoiceMemo", &format!("発話開始検出 (RMS={:.4})", rms));
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
                    self.send_speech_chunk(resources);
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
    fn send_speech_chunk(&mut self, resources: &Resources) {
        let Some(recorder) = &resources.voice_recorder else {
            return;
        };

        // 発話区間のサンプルを取得
        let slice = recorder.get_samples_since(self.speech_start_sample);
        let samples = slice.samples;

        // 送信に必要な最小サンプル数
        let min_speech_samples = (SAMPLE_RATE as f32 * MIN_SPEECH_SECS) as usize;

        if samples.len() < min_speech_samples {
            let duration_secs = samples.len() as f32 / SAMPLE_RATE as f32;
            log_debug!(
                "VoiceMemo",
                &format!(
                    "発話が短すぎるためスキップ ({:.1}秒, {}サンプル)",
                    duration_secs,
                    samples.len()
                )
            );
            self.reset_vad_state();
            return;
        }

        let duration_secs = samples.len() as f32 / SAMPLE_RATE as f32;
        log_debug!(
            "VoiceMemo",
            &format!(
                "発話チャンク準備: {:.1}秒分 ({}サンプル)",
                duration_secs,
                samples.len()
            )
        );

        self.send_transcription_request(samples, slice.start);
        self.reset_vad_state();
    }

    /// VAD状態をリセット
    pub(super) fn reset_vad_state(&mut self) {
        self.is_speaking = false;
        self.silence_start = None;
        self.speech_start_sample = self.last_vad_check_sample;
    }
}
