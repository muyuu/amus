//! 発話区間の検出
//!
//! 録音の音量を窓単位で与えると、書き起こしへ送る区間を返す。判定は録音上のサンプル
//! 位置だけで行い、実時間には依存しない。フレームレートの揺れが区間に乗らないように。

use crate::voice_recorder::SAMPLE_RATE;
use std::ops::Range;

/// 発話を開始と判定する、暗騒音に対する倍率。
///
/// 小声だと暗騒音のすぐ上でしか喋らない。取り逃すと発言そのものが失われる一方、
/// 拾いすぎても書き起こしが1回無駄になるだけなので、低めに置く。
const OPEN_RATIO: f32 = 2.0;
/// 発話の継続と判定する倍率。開始より低くして、語中の息継ぎで切らない。
const KEEP_RATIO: f32 = 1.3;
/// 暗騒音がほぼ無い環境で、わずかな雑音を発話と見なさないための下限。
const ABSOLUTE_FLOOR: f32 = 0.003;
/// 暗騒音が下がったときの追従率。速く下げる。
const NOISE_FALL: f32 = 0.5;
/// 暗騒音が上がるときの追従率。発話で引き上げられないよう、ごく緩やかにする。
const NOISE_RISE: f32 = 0.002;

/// 発話終了とみなす無音の長さ。
const SILENCE_SECS: f32 = 1.0;
/// 発話開始より前に遡って含める長さ。立ち上がりの短い子音を落とさない。
const PRE_ROLL_SECS: f32 = 0.3;
/// 発話終了後に含める長さ。語尾を落とさない。
const POST_ROLL_SECS: f32 = 0.3;

fn samples(secs: f32) -> usize {
    (SAMPLE_RATE as f32 * secs) as usize
}

/// 発話区間の検出器。
///
/// 閾値は暗騒音からの相対で決める。マイクの入力レベルは環境ごとに大きく違い、
/// 固定値では音量の小さい環境で発話を取り逃す。
#[derive(Default)]
pub struct SpeechDetector {
    /// 暗騒音の推定値。最初に観測した音量で初期化する。
    ///
    /// 固定値から始めると、静かな環境では発話まで上がりきらず取り逃し、うるさい
    /// 環境では暗騒音を発話と取り違える。録音開始直後は無音なのが普通なので、
    /// 最初の観測をそのまま部屋の暗さとして採る。
    noise_floor: Option<f32>,
    /// 発話中なら、その区間の開始位置（前に遡った分を含む）
    speech_start: Option<usize>,
    /// 最後に音を捉えた窓の終端
    last_voiced_end: usize,
}

impl SpeechDetector {
    /// 窓ひとつ分の音量を与える。発話区間が確定したらその範囲を返す。
    ///
    /// `window` はその窓が占める録音上の範囲、`rms` はその区間の音量。
    pub fn observe(&mut self, window: Range<usize>, rms: f32) -> Option<Range<usize>> {
        let floor = *self.noise_floor.get_or_insert(rms);

        let voiced = match self.speech_start {
            // 開始は高い閾値で判定する。暗騒音を発話と取り違えない。
            None => rms > (floor * OPEN_RATIO).max(ABSOLUTE_FLOOR),
            // 継続は低い閾値で判定する。語中の息継ぎで切らない。
            Some(_) => rms > (floor * KEEP_RATIO).max(ABSOLUTE_FLOOR),
        };

        self.noise_floor = Some(if rms < floor {
            floor + (rms - floor) * NOISE_FALL
        } else {
            floor + (rms - floor) * NOISE_RISE
        });

        if voiced {
            self.speech_start
                .get_or_insert_with(|| window.start.saturating_sub(samples(PRE_ROLL_SECS)));
            self.last_voiced_end = window.end;
            return None;
        }

        let start = self.speech_start?;
        if window.end - self.last_voiced_end < samples(SILENCE_SECS) {
            return None;
        }

        // 後ろの無音は落とす。無音を長く渡すとモデルが埋めに行く。
        let end = self.last_voiced_end + samples(POST_ROLL_SECS);
        self.speech_start = None;
        Some(start..end)
    }

    /// 未確定のまま進行中の発話があれば、その開始位置。
    ///
    /// 録音が止まるときに、言いかけを取りこぼさないために使う。
    pub fn pending_speech(&self) -> Option<usize> {
        self.speech_start
    }

    /// 録音を取り直したときに、位置の対応を捨てる。
    pub fn reset(&mut self) {
        self.speech_start = None;
        self.last_voiced_end = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: usize = 1600; // 0.1 秒

    /// 音量の並びを 0.1 秒刻みで流し込み、確定した区間を集める。
    fn run(levels: &[f32]) -> Vec<Range<usize>> {
        let mut detector = SpeechDetector::default();
        levels
            .iter()
            .enumerate()
            .filter_map(|(i, &rms)| detector.observe(i * WINDOW..(i + 1) * WINDOW, rms))
            .collect()
    }

    /// `secs` 秒分の同じ音量。
    fn level(rms: f32, secs: f32) -> Vec<f32> {
        vec![rms; (secs / 0.1).round() as usize]
    }

    fn sequence(parts: &[Vec<f32>]) -> Vec<f32> {
        parts.concat()
    }

    #[test]
    fn a_short_pause_inside_a_word_does_not_end_the_speech() {
        // 語中の息継ぎで切ると語尾が落ちる
        let levels = sequence(&[
            level(0.004, 0.5), // 暗騒音
            level(0.02, 0.8),  // 発話
            level(0.006, 0.2), // 息継ぎ
            level(0.02, 0.8),  // 発話の続き
            level(0.004, 1.5), // 無音
        ]);

        let found = run(&levels);

        assert_eq!(found.len(), 1, "息継ぎで区切ってはいけない: {:?}", found);
    }

    #[test]
    fn the_chunk_starts_before_the_first_loud_window() {
        // 「コ」のような立ち上がりの短い子音は閾値を超えるまでに遅れる
        let levels = sequence(&[level(0.004, 1.0), level(0.02, 1.0), level(0.004, 1.5)]);

        let found = run(&levels);
        let speech_began = samples(1.0);

        assert!(
            found[0].start < speech_began,
            "立ち上がりより前から切り出すこと: {:?}",
            found[0]
        );
    }

    #[test]
    fn the_chunk_keeps_the_tail_after_the_last_loud_window() {
        let levels = sequence(&[level(0.004, 0.5), level(0.02, 1.0), level(0.004, 1.5)]);

        let found = run(&levels);
        let speech_ended = samples(1.5);

        assert!(
            found[0].end > speech_ended,
            "語尾より後まで含めること: {:?}",
            found[0]
        );
    }

    #[test]
    fn quiet_speech_is_detected_when_the_room_is_quiet() {
        // 検証環境のマイクは発話時 RMS が 0.010 前後しかない
        let levels = sequence(&[level(0.002, 1.0), level(0.010, 1.0), level(0.002, 1.5)]);

        assert_eq!(run(&levels).len(), 1, "小さい声を取り逃してはいけない");
    }

    #[test]
    fn a_soft_voice_is_detected_over_real_room_noise() {
        // 実測の環境。暗騒音のすぐ上でしか喋っていない
        let levels = sequence(&[level(0.004, 1.0), level(0.010, 1.2), level(0.004, 1.5)]);

        assert_eq!(run(&levels).len(), 1, "小声を取り逃してはいけない");
    }

    #[test]
    fn background_noise_alone_is_not_speech() {
        let levels = sequence(&[level(0.008, 3.0)]);

        assert!(run(&levels).is_empty(), "暗騒音だけで発話としない");
    }
}
