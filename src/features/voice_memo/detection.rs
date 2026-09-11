//! ホットワード検知の駆動
//!
//! 録音中の音声を一定間隔で切り出して検知スレッドへ送り、結果からターン境界を操作する。

use super::hotword::TurnBoundary;
use super::VoiceMemoFeature;
use crate::log_debug;
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::{HotwordRequest, Resources};

/// 検知窓の長さ。トリガーワード全体が収まる程度に取る。
const WINDOW_SECS: f32 = 2.0;

/// 窓を送る間隔。窓長より短くして重なりを持たせ、境界にまたがる語を取り逃さない。
const STRIDE_SECS: f32 = 1.0;

impl VoiceMemoFeature {
    /// 検知窓を送る。処理が追いつかないうちは送らない。
    pub(super) fn send_hotword_window(&mut self, resources: &Resources) {
        let (Some(thread), Some(recorder)) = (&self.hotword_thread, &resources.voice_recorder)
        else {
            return;
        };

        // 前の窓を処理中なら待つ。溜め込むと検知が実時間から遅れていく。
        if self.hotword_pending {
            return;
        }

        let window_samples = (SAMPLE_RATE as f32 * WINDOW_SECS) as usize;
        let stride_samples = (SAMPLE_RATE as f32 * STRIDE_SECS) as usize;

        let now = recorder.buffer_len();
        if now < window_samples || now < self.last_hotword_end + stride_samples {
            return;
        }

        let slice = recorder.get_samples_since(now - window_samples);
        thread.request(HotwordRequest {
            samples: slice.samples,
            window_end_sample: now,
        });
        self.hotword_pending = true;
        self.last_hotword_end = now;
    }

    /// 検知結果を取り込み、ターン境界を操作する。
    pub(super) fn poll_hotword_results(&mut self) {
        let Some(thread) = &self.hotword_thread else {
            return;
        };

        for result in thread.poll_results() {
            self.hotword_pending = false;

            let Some(hotword) = self.matcher.find(&result.text) else {
                continue;
            };

            // 窓の終端を境界にする。トリガーワードの発話そのものをターンに含めないため。
            let at = result.window_end_sample;
            let Some(boundary) = self
                .boundary_tracker
                .accept(hotword, at, self.state.round_active)
            else {
                continue;
            };

            log_debug!(
                "VoiceMemo",
                &format!(
                    "ホットワード検知: {:?} -> {:?} at {}",
                    hotword, boundary, at
                )
            );

            match boundary {
                TurnBoundary::Open => self.open_turn(at),
                TurnBoundary::Close => self.close_turn(at),
                TurnBoundary::Restart => {
                    self.close_turn(at);
                    self.open_turn(at);
                }
            }
        }
    }
}

/// 既定のトリガーワード。
///
/// 漢字とかなの違いは照合で吸収できないため、想定する表記を並べる。
pub(super) const DEFAULT_START_WORDS: &[&str] = &["ターン開始", "ターンかいし"];
pub(super) const DEFAULT_END_WORDS: &[&str] = &["ターン終了", "ターンしゅうりょう"];

/// 同じ発話を重ねて拾わないためのクールダウン。
pub(super) const COOLDOWN_SECS: f32 = 3.0;
