//! 逐次リサンプリング
//!
//! 録音コールバックは音声を細切れのチャンクで受け取る。チャンクごとに独立して
//! 変換すると境界で位相がずれ、レートが少しずつ狂う。位相をチャンクをまたいで
//! 持ち越すため、状態を持つ。

/// 線形補間でサンプルレートを変換する。
///
/// 入力チャンクを順に与えると、変換後のサンプルを順に返す。チャンクの切れ目に
/// 依存しない結果になる。
pub struct Resampler {
    /// 出力1サンプルあたりに進む入力サンプル数。
    step: f64,
    /// 次に出力する位置（現在のチャンク先頭を 0 とした入力サンプル単位）。
    pos: f64,
    /// 直前の入力サンプル。チャンクをまたいだ補間に使う。
    prev: f32,
}

impl Resampler {
    /// `src_rate` から `dst_rate` へ変換するリサンプラを作る。
    pub fn new(src_rate: u32, dst_rate: u32) -> Self {
        Self {
            step: src_rate as f64 / dst_rate as f64,
            pos: 0.0,
            prev: 0.0,
        }
    }

    /// 入力チャンクを変換し、出力サンプルを `sink` に渡す。
    pub fn process(&mut self, src: &[f32], mut sink: impl FnMut(f32)) {
        for (index, &sample) in src.iter().enumerate() {
            let index = index as f64;
            while self.pos <= index {
                // pos は直前の入力サンプルと今の入力サンプルの間にある。
                let t = self.pos - (index - 1.0);
                sink((self.prev as f64 * (1.0 - t) + sample as f64 * t) as f32);
                self.pos += self.step;
            }
            self.prev = sample;
        }
        // 次のチャンクの先頭を 0 とする位置へ繰り越す。
        self.pos -= src.len() as f64;
    }
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

        let out = collect(&mut resampler, &[1.0, 2.0, 3.0]);

        assert_eq!(out, vec![1.0, 2.0, 3.0]);
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
        let src = vec![0.0f32; 480]; // 48kHz で 10ms

        let out = collect(&mut resampler, &src);

        // 16kHz の 10ms = 160 サンプル
        assert_eq!(out.len(), 160);
    }
}
