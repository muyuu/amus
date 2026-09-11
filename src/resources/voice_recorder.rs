//! 音声録音リソース
//!
//! マイクからの音声入力を録音し、16kHz mono f32 形式で出力する。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use thiserror::Error;

/// 録音バッファに保持する秒数の上限。
///
/// 呼び出し側が破棄を忘れても際限なく増えないための上限。ネイティブの
/// サンプルレート・f32 のまま保持するため、48kHz では 1 秒あたり約 192KB
/// 消費する（この上限で約 57MB）。
const BUFFER_CAPACITY_SECS: usize = 300;

/// poison したロックも回復して使う。
///
/// 録音バッファと開始時刻はオーディオコールバックスレッドと共有する。
/// コールバックが panic してロックが poison しても、途中までの録音データは
/// そのまま使い続けて差し支えなく、ここで panic させてアプリ全体を巻き込む
/// 方が害が大きい。よって poison は無視して継続する。
fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 録音サンプルを、確定送出済みの先頭領域を破棄しつつ保持するバッファ。
///
/// 呼び出し側は VAD の保持位置（`speech_start_sample` など）を **絶対インデックス**
/// として扱う。先頭を破棄しても絶対インデックスは変えないため、`base`（破棄済み
/// サンプル数 = `samples[0]` の絶対位置）を保持し、内部で相対位置へ換算する。
/// これにより長時間録音でバッファが単調増加するのを防ぐ。
#[derive(Default)]
struct RecordingBuffer {
    /// 先頭の破棄を O(1) で行うため `VecDeque`。録音コールバックから毎サンプル
    /// 呼ばれるので、`Vec` の先頭 drain（全要素の移動）は使えない。
    samples: VecDeque<f32>,
    /// 破棄済みサンプル数（= `samples[0]` の絶対インデックス）。
    base: usize,
    /// 保持するサンプル数の上限。0 は無制限。
    capacity: usize,
}

impl RecordingBuffer {
    /// 保持上限を指定して作る。上限を超えた分は古い側から落とす。
    ///
    /// ゲーム開始から録音し続けるため、呼び出し側の破棄に頼らず上限を設ける。
    fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            ..Default::default()
        }
    }

    fn clear(&mut self) {
        self.samples.clear();
        self.base = 0;
    }

    fn push(&mut self, sample: f32) {
        self.samples.push_back(sample);
        self.enforce_capacity();
    }

    /// 上限を超えた分を古い側から落とす。絶対インデックスは変えない。
    fn enforce_capacity(&mut self) {
        if self.capacity == 0 {
            return;
        }
        while self.samples.len() > self.capacity {
            self.samples.pop_front();
            self.base += 1;
        }
    }

    /// 絶対長（破棄分を含む、これまでに録音した総サンプル数）。
    fn len_abs(&self) -> usize {
        self.base + self.samples.len()
    }

    /// `abs_start` 以降の残っているサンプルを返す。
    ///
    /// 保持上限を超えて古い側が失われている場合、返るのは残っている分だけで、
    /// `RecordedSlice::start` が実際の先頭位置を示す。
    fn samples_since(&self, abs_start: usize) -> RecordedSlice {
        if abs_start >= self.len_abs() {
            return RecordedSlice {
                samples: Vec::new(),
                start: abs_start,
            };
        }
        let start = abs_start.max(self.base);
        let rel = start - self.base;
        RecordedSlice {
            samples: self.samples.range(rel..).copied().collect(),
            start,
        }
    }

    /// `abs_index` より前の確定領域を破棄する（絶対インデックスは不変）。
    fn discard_before(&mut self, abs_index: usize) {
        let rel = abs_index.saturating_sub(self.base).min(self.samples.len());
        self.samples.drain(..rel);
        self.base += rel;
    }
}

/// バッファから読み出したサンプルと、その絶対位置。
pub struct RecordedSlice {
    pub samples: Vec<f32>,
    /// 実際に返した先頭サンプルの絶対位置。保持上限を超えて古い側が失われた場合、
    /// 要求した位置より後ろになる。タイムスタンプの計算にはこちらを使うこと。
    pub start: usize,
}

/// 音声録音リソースの操作が失敗した理由。
#[derive(Debug, Error)]
pub enum RecorderError {
    #[error("マイクが見つかりません")]
    NoInputDevice,
    #[error("入力設定の取得に失敗: {0}")]
    Config(#[from] cpal::DefaultStreamConfigError),
    #[error("未対応のサンプルフォーマット: {0:?}")]
    UnsupportedFormat(SampleFormat),
    #[error("録音ストリームの作成に失敗: {0}")]
    BuildStream(#[from] cpal::BuildStreamError),
    #[error("録音開始に失敗: {0}")]
    Play(#[from] cpal::PlayStreamError),
    #[error("WAV の保存に失敗: {0}")]
    Wav(#[from] hound::Error),
}

/// 音声録音を管理
pub struct VoiceRecorder {
    device: Device,
    config: StreamConfig,
    sample_format: SampleFormat,
    sample_rate: u32,
    /// 録音中の音声データバッファ
    buffer: Arc<Mutex<RecordingBuffer>>,
    /// 録音開始時刻
    start_time: Arc<Mutex<Option<Instant>>>,
    /// 録音ストリーム
    stream: Option<Stream>,
}

impl VoiceRecorder {
    /// 新しいVoiceRecorderを作成
    pub fn new() -> Result<Self, RecorderError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(RecorderError::NoInputDevice)?;

        let supported_config = device.default_input_config()?;

        let sample_format = supported_config.sample_format();
        let sample_rate = supported_config.sample_rate();
        let config: StreamConfig = supported_config.into();

        crate::log_debug!(
            "VoiceRecorder",
            format!(
                "sample_format={:?}, sample_rate={}, channels={}",
                sample_format, sample_rate, config.channels
            )
        );

        Ok(Self {
            device,
            config,
            sample_format,
            sample_rate,
            buffer: Arc::new(Mutex::new(RecordingBuffer::with_capacity(
                sample_rate as usize * BUFFER_CAPACITY_SECS,
            ))),
            start_time: Arc::new(Mutex::new(None)),
            stream: None,
        })
    }

    /// 録音を開始
    pub fn start_recording(&mut self) -> Result<(), RecorderError> {
        // バッファをクリア
        {
            let mut buffer = lock_recover(&self.buffer);
            buffer.clear();
        }

        // 開始時刻を記録
        {
            let mut start_time = lock_recover(&self.start_time);
            *start_time = Some(Instant::now());
        }

        let channels = self.config.channels as usize;
        let err_fn = |err| {
            crate::log_error!("VoiceRecorder", format!("録音エラー: {}", err));
        };

        let stream = match self.sample_format {
            SampleFormat::I16 => {
                let buffer = Arc::clone(&self.buffer);
                self.device.build_input_stream(
                    &self.config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buffer = lock_recover(&buffer);
                        for chunk in data.chunks(channels) {
                            if let Some(&sample) = chunk.first() {
                                // i16 を f32 に変換 (-1.0 ~ 1.0)
                                buffer.push(sample as f32 / i16::MAX as f32);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::I32 => {
                let buffer = Arc::clone(&self.buffer);
                self.device.build_input_stream(
                    &self.config,
                    move |data: &[i32], _: &cpal::InputCallbackInfo| {
                        let mut buffer = lock_recover(&buffer);
                        for chunk in data.chunks(channels) {
                            if let Some(&sample) = chunk.first() {
                                // i32 を f32 に変換 (-1.0 ~ 1.0)
                                buffer.push(sample as f32 / i32::MAX as f32);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::F32 => {
                let buffer = Arc::clone(&self.buffer);
                self.device.build_input_stream(
                    &self.config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buffer = lock_recover(&buffer);
                        for chunk in data.chunks(channels) {
                            if let Some(&sample) = chunk.first() {
                                buffer.push(sample);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            _ => {
                return Err(RecorderError::UnsupportedFormat(self.sample_format));
            }
        };

        stream.play()?;

        self.stream = Some(stream);
        Ok(())
    }

    /// 録音を停止する
    pub fn stop_recording(&mut self) {
        // ストリームを停止
        self.stream = None;

        let total_samples = lock_recover(&self.buffer).len_abs();

        let duration_secs = {
            let start_time = lock_recover(&self.start_time);
            start_time
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(0.0)
        };

        crate::log_debug!(
            "VoiceRecorder",
            format!(
                "録音停止: {}サンプル取得, 録音時間={:.1}秒, 元サンプルレート={}",
                total_samples, duration_secs, self.sample_rate
            )
        );
    }

    /// 現在の録音時間（秒）
    pub fn elapsed_secs(&self) -> f32 {
        let start_time = lock_recover(&self.start_time);
        if let Some(start) = *start_time {
            start.elapsed().as_secs_f32()
        } else {
            0.0
        }
    }

    /// これまでに録音した総サンプル数（破棄済みを含む絶対長）
    pub fn buffer_len(&self) -> usize {
        lock_recover(&self.buffer).len_abs()
    }

    /// 元のサンプルレートを取得
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 指定位置（絶対インデックス）以降のサンプルを取得（録音を停止せずに）。
    ///
    /// 逐次書き起こし用。サンプルは 16kHz にリサンプリング済み。位置は元の
    /// サンプルレートでの絶対インデックスのまま返す。
    ///
    /// 保持上限を超えて古い側が失われている場合、`RecordedSlice::start` が
    /// 要求位置より後ろになる。
    pub fn get_samples_since(&self, start_sample: usize) -> RecordedSlice {
        let slice = {
            let buffer = lock_recover(&self.buffer);
            buffer.samples_since(start_sample)
        };

        RecordedSlice {
            samples: self.resample_to_16k(&slice.samples),
            ..slice
        }
    }

    /// `keep_from`（絶対インデックス）より前の確定送出済み領域を破棄する。
    /// 録音を止めずにバッファの単調増加を抑える。絶対インデックスは変わらない。
    pub fn discard_before(&self, keep_from: usize) {
        lock_recover(&self.buffer).discard_before(keep_from);
    }

    /// サンプルレートを16kHzにリサンプリング
    fn resample_to_16k(&self, samples: &[f32]) -> Vec<f32> {
        const TARGET_RATE: u32 = 16000;

        if self.sample_rate == TARGET_RATE {
            return samples.to_vec();
        }

        // 簡易的な線形補間リサンプリング
        let ratio = self.sample_rate as f64 / TARGET_RATE as f64;
        let new_len = (samples.len() as f64 / ratio) as usize;
        let mut resampled = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = i as f64 * ratio;
            let idx_floor = src_idx.floor() as usize;
            let idx_ceil = (idx_floor + 1).min(samples.len() - 1);
            let frac = src_idx - idx_floor as f64;

            let sample = samples[idx_floor] as f64 * (1.0 - frac) + samples[idx_ceil] as f64 * frac;
            resampled.push(sample as f32);
        }

        resampled
    }

    /// 録音データをWAVファイルとして保存（デバッグ用）
    #[allow(dead_code)]
    pub fn save_to_wav(&self, samples: &[f32], path: &str) -> Result<(), RecorderError> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        for &sample in samples {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn buffer_keeps_absolute_indices_across_discard() {
        let mut buf = RecordingBuffer::default();
        for i in 0..10 {
            buf.push(i as f32);
        }
        assert_eq!(buf.len_abs(), 10);

        // 絶対インデックス 4 以降を取得
        let slice = buf.samples_since(4);
        assert_eq!(slice.samples, vec![4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);

        // 先頭 4 サンプルを破棄しても絶対インデックスは不変
        buf.discard_before(4);
        assert_eq!(buf.len_abs(), 10);
        assert_eq!(buf.samples.len(), 6); // 実メモリは縮む
        let slice2 = buf.samples_since(4);
        assert_eq!(slice2.samples, vec![4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);

        // さらに録音が進んでも整合する
        buf.push(10.0);
        let slice3 = buf.samples_since(8);
        assert_eq!(slice3.samples, vec![8.0, 9.0, 10.0]);
    }

    #[test]
    fn buffer_drops_oldest_samples_beyond_capacity() {
        let mut buf = RecordingBuffer::with_capacity(4);
        for i in 0..6 {
            buf.push(i as f32);
        }

        // 絶対インデックスは録音開始からの通し番号のまま
        assert_eq!(buf.len_abs(), 6);
        // 実メモリは上限に収まり、新しい側が残る
        assert_eq!(buf.samples.len(), 4);
        let slice = buf.samples_since(2);
        assert_eq!(slice.samples, vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn buffer_reports_actual_start_when_requested_position_was_dropped() {
        let mut buf = RecordingBuffer::with_capacity(4);
        for i in 0..6 {
            buf.push(i as f32);
        }

        // 絶対位置 0,1 は容量超過で失われている
        let slice = buf.samples_since(0);

        assert_eq!(slice.start, 2, "失われた分だけ開始位置が後ろへずれる");
        assert_eq!(slice.samples, vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn buffer_samples_since_past_end_is_empty() {
        let mut buf = RecordingBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        let slice = buf.samples_since(5);
        assert!(slice.samples.is_empty());
    }

    #[test]
    fn buffer_discard_before_already_discarded_is_noop() {
        let mut buf = RecordingBuffer::default();
        for i in 0..5 {
            buf.push(i as f32);
        }
        buf.discard_before(3);
        // 既に破棄済みより前を指しても壊れない
        buf.discard_before(1);
        assert_eq!(buf.base, 3);
        let slice = buf.samples_since(3);
        assert_eq!(slice.samples, vec![3.0, 4.0]);
    }

    #[test]
    fn lock_recover_returns_inner_after_poison() {
        let m = Arc::new(Mutex::new(vec![1.0f32, 2.0]));

        // 別スレッドがロック保持中に panic し、mutex を poison させる
        let m2 = Arc::clone(&m);
        let _ = std::thread::spawn(move || {
            let _guard = m2.lock().unwrap();
            panic!("poison");
        })
        .join();

        // poison していても panic せず中身を読める
        let guard = lock_recover(&m);
        assert_eq!(&*guard, &[1.0, 2.0]);
    }
}
