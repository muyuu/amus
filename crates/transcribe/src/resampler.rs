//! 逐次リサンプリング
//!
//! 録音コールバックは音声を細切れのチャンクで受け取る。チャンクごとに独立して
//! 変換すると境界で位相がずれ、レートが少しずつ狂う。位相と履歴をチャンクをまたいで
//! 持ち越すため、状態を持つ。

use std::collections::VecDeque;
use std::f64::consts::PI;

/// 補間に使うタップ数の片側。多いほど遮断が急になる。
const HALF_TAPS: i64 = 32;

/// 窓関数付き sinc 補間でサンプルレートを変換する。
///
/// ダウンサンプリングでは、間引く前に出力側のナイキストより上を落とす必要がある。
/// 落とさずに間引くと高域がそのまま可聴帯域へ折り返り、高域にエネルギーを持つ
/// 子音が潰れる。sinc の帯域を出力側のナイキストに合わせることで、補間と遮断を
/// 同時に行う。
///
/// 補間には出力位置より後ろのサンプルが要るため、出力は入力に対して
/// `HALF_TAPS` サンプル分遅れる。
pub struct Resampler {
    /// 出力1サンプルあたりに進む入力サンプル数。
    step: f64,
    /// 遮断周波数。入力のナイキストを 1.0 とする。
    cutoff: f64,
    /// 次に出力する位置（入力サンプル単位の絶対位置）。
    pos: f64,
    /// 補間に必要な範囲の入力。
    history: VecDeque<f32>,
    /// `history` の先頭が入力の何サンプル目か。
    offset: i64,
}

impl Resampler {
    /// `src_rate` から `dst_rate` へ変換するリサンプラを作る。
    pub fn new(src_rate: u32, dst_rate: u32) -> Self {
        let step = src_rate as f64 / dst_rate as f64;

        Self {
            step,
            // アップサンプリングでは入力の帯域がそのまま上限になる
            cutoff: (1.0 / step).min(1.0),
            pos: 0.0,
            history: VecDeque::new(),
            offset: 0,
        }
    }

    /// 入力チャンクを変換し、出力サンプルを `sink` に渡す。
    pub fn process(&mut self, src: &[f32], mut sink: impl FnMut(f32)) {
        self.history.extend(src.iter().copied());

        // 中心より後ろのタップが揃った位置まで出力する
        let available = self.offset + self.history.len() as i64;
        while self.pos + HALF_TAPS as f64 <= available as f64 {
            sink(self.interpolate(self.pos));
            self.pos += self.step;
        }

        self.drop_used_history();
    }

    /// `pos` の位置を周囲のサンプルから求める。
    fn interpolate(&self, pos: f64) -> f32 {
        let center = pos.floor() as i64;
        let mut sum = 0.0;
        let mut norm = 0.0;

        for index in (center - HALF_TAPS + 1)..=(center + HALF_TAPS) {
            let Some(sample) = self.sample(index) else {
                continue;
            };

            let distance = pos - index as f64;
            let weight = sinc(distance * self.cutoff) * window(distance);
            sum += sample as f64 * weight;
            norm += weight;
        }

        // 端では一部のタップが欠けるため、重みの合計で割って利得を揃える
        if norm.abs() < f64::EPSILON {
            0.0
        } else {
            (sum / norm) as f32
        }
    }

    /// 入力の `index` サンプル目。まだ来ていない・もう捨てた場合は `None`。
    fn sample(&self, index: i64) -> Option<f32> {
        let at = index - self.offset;
        if at < 0 {
            return None;
        }
        self.history.get(at as usize).copied()
    }

    /// 次の出力にもう使わない履歴を捨てる。
    fn drop_used_history(&mut self) {
        let needed_from = self.pos.floor() as i64 - HALF_TAPS;

        while self.offset < needed_from && self.history.pop_front().is_some() {
            self.offset += 1;
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// ブラックマン窓。`distance` は中心からの入力サンプル数。
fn window(distance: f64) -> f64 {
    let half = HALF_TAPS as f64;
    if distance.abs() > half {
        return 0.0;
    }

    let phase = (distance + half) / (2.0 * half);
    0.42 - 0.5 * (2.0 * PI * phase).cos() + 0.08 * (4.0 * PI * phase).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(resampler: &mut Resampler, src: &[f32]) -> Vec<f32> {
        let mut out = Vec::new();
        resampler.process(src, |s| out.push(s));
        out
    }

    #[test]
    fn same_rate_passes_samples_through() {
        let mut resampler = Resampler::new(16000, 16000);
        let src: Vec<f32> = (0..128).map(|i| i as f32).collect();

        let out = collect(&mut resampler, &src);

        assert!(!out.is_empty());
        for (index, &value) in out.iter().enumerate() {
            assert!(
                (value - index as f32).abs() < 1e-3,
                "{index}番目が {value} になった"
            );
        }
    }

    #[test]
    fn chunk_boundaries_do_not_change_the_result() {
        let src: Vec<f32> = (0..300).map(|i| (i as f32 * 0.01).sin()).collect();

        let mut whole = Resampler::new(48000, 16000);
        let expected = collect(&mut whole, &src);

        // 割り切れない長さで分割しても同じ結果になること
        let mut chunked = Resampler::new(48000, 16000);
        let mut actual = Vec::new();
        for chunk in src.chunks(7) {
            chunked.process(chunk, |s| actual.push(s));
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn downsampling_keeps_the_output_rate() {
        let mut resampler = Resampler::new(48000, 16000);
        let src = vec![0.0f32; 48000]; // 48kHz で1秒

        let out = collect(&mut resampler, &src);

        // 16kHz の1秒 = 16000サンプル。先読みの分だけ届かない。
        assert!(
            (15_980..=16_000).contains(&out.len()),
            "出力レートがずれている: {}",
            out.len()
        );
    }
}

#[cfg(test)]
mod antialias_tests {
    use super::*;

    fn sine(freq: f64, rate: u32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 / rate as f64).sin() as f32)
            .collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    }

    fn resample(freq: f64) -> Vec<f32> {
        let src = sine(freq, 48_000, 48_000);
        let mut resampler = Resampler::new(48_000, 16_000);
        let mut out = Vec::new();
        resampler.process(&src, |s| out.push(s));
        out
    }

    #[test]
    fn a_tone_above_the_output_nyquist_is_removed() {
        // 16kHz のナイキストは 8kHz。落とさずに間引くと 12kHz は 4kHz へ折り返す。
        let out = resample(12_000.0);

        assert!(rms(&out) < 0.1, "折り返しが残っている: rms={}", rms(&out));
    }

    #[test]
    fn a_tone_within_the_output_band_passes_through() {
        let out = resample(1_000.0);

        // 正弦波の RMS は 1/√2 ≒ 0.707
        assert!(rms(&out) > 0.6, "通過帯域が減衰している: rms={}", rms(&out));
    }
}
