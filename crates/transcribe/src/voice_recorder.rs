//! 音声録音リソース
//!
//! マイクからの音声入力を録音し、16kHz mono f32 形式で出力する。

use super::resampler::Resampler;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use thiserror::Error;

/// このレコーダが返すサンプルと、絶対インデックスの単位となるレート。
///
/// デバイスのレートに関わらずここへ揃える。Whisper が 16kHz を要求するのに
/// 加え、絶対インデックスがそのまま時間（`index / SAMPLE_RATE` 秒）になる。
pub const SAMPLE_RATE: u32 = 16000;

/// 録音バッファに保持する秒数の上限。
///
/// 呼び出し側が破棄を忘れても際限なく増えないための上限。16kHz・i16 で保持する
/// ため 1 秒あたり 32KB 消費する（この上限で約 58MB）。
const BUFFER_CAPACITY_SECS: usize = 1800;

/// poison したロックも回復して使う。
///
/// 録音バッファはオーディオコールバックスレッドと共有する。
/// コールバックが panic してロックが poison しても、途中までの録音データは
/// そのまま使い続けて差し支えなく、ここで panic させてアプリ全体を巻き込む
/// 方が害が大きい。よって poison は無視して継続する。
fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 録音サンプルを、上限まで保持するリングバッファ。
///
/// 呼び出し側は保持位置（`speech_start_sample` など）を **絶対インデックス**
/// として扱う。上限を超えて先頭が失われても絶対インデックスは変えないため、
/// `base`（失われたサンプル数 = `samples[0]` の絶対位置）を保持し、内部で
/// 相対位置へ換算する。
#[derive(Default)]
struct RecordingBuffer {
    /// 先頭の破棄を O(1) で行うため `VecDeque`。録音コールバックから毎サンプル
    /// 呼ばれるので、`Vec` の先頭 drain（全要素の移動）は使えない。
    ///
    /// 長時間の保持がメモリを圧迫するため、f32 ではなく i16 で持つ。
    samples: VecDeque<i16>,
    /// 失われたサンプル数（= `samples[0]` の絶対インデックス）。
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

    /// 保持しているサンプルを捨てる。絶対位置は巻き戻さない。
    ///
    /// 絶対位置はレコーダーの生涯で単調増加する。録音を取り直したときに位置が
    /// 振り直されると、前の録音を指す位置が新しい録音の位置と衝突するため。
    fn clear(&mut self) {
        self.base = self.len_abs();
        self.samples.clear();
    }

    /// 範囲外の値は ±1.0 に丸めて格納する。
    fn push(&mut self, sample: f32) {
        self.samples.push_back(f32_to_i16(sample));
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

    /// 絶対長（失われた分を含む、これまでに録音した総サンプル数）。
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
            samples: self.samples.range(rel..).copied().map(i16_to_f32).collect(),
            start,
        }
    }
}

/// 録音コールバックを作る。
///
/// デバイスから届くインターリーブされたサンプルの先頭チャンネルだけを取り、
/// `SAMPLE_RATE` へ変換してバッファへ積む。リサンプラはチャンクをまたいで
/// 位相を保つ必要があるため、クロージャが保持する。
fn capture<T: Copy>(
    buffer: Arc<Mutex<RecordingBuffer>>,
    channels: usize,
    device_rate: u32,
    to_f32: fn(T) -> f32,
) -> impl FnMut(&[T], &cpal::InputCallbackInfo) {
    let mut resampler = Resampler::new(device_rate, SAMPLE_RATE);
    // コールバックごとの確保を避けるため使い回す。
    let mut mono: Vec<f32> = Vec::new();

    move |data: &[T], _: &cpal::InputCallbackInfo| {
        mono.clear();
        mono.extend(
            data.chunks(channels)
                .filter_map(|frame| frame.first().map(|&s| to_f32(s))),
        );

        let mut buffer = lock_recover(&buffer);
        resampler.process(&mono, |sample| buffer.push(sample));
    }
}

/// -1.0〜1.0 を i16 の全域へ写す。範囲外は丸める。
fn f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

fn i16_to_f32(sample: i16) -> f32 {
    sample as f32 / i16::MAX as f32
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
    /// デバイス側のサンプルレート。リサンプラの入力レートとしてのみ使う。
    device_sample_rate: u32,
    /// 録音中の音声データバッファ
    buffer: Arc<Mutex<RecordingBuffer>>,
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
        let device_sample_rate = supported_config.sample_rate();
        let config: StreamConfig = supported_config.into();

        crate::log_debug!(
            "VoiceRecorder",
            format!(
                "sample_format={:?}, sample_rate={}, channels={}",
                sample_format, device_sample_rate, config.channels
            )
        );

        Ok(Self {
            device,
            config,
            sample_format,
            device_sample_rate,
            buffer: Arc::new(Mutex::new(RecordingBuffer::with_capacity(
                SAMPLE_RATE as usize * BUFFER_CAPACITY_SECS,
            ))),
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

        let channels = self.config.channels as usize;
        let err_fn = |err| {
            crate::log_error!("VoiceRecorder", format!("録音エラー: {}", err));
        };

        let device_rate = self.device_sample_rate;
        let stream = match self.sample_format {
            SampleFormat::I16 => {
                let buffer = Arc::clone(&self.buffer);
                self.device.build_input_stream(
                    &self.config,
                    capture(buffer, channels, device_rate, |s: i16| {
                        s as f32 / i16::MAX as f32
                    }),
                    err_fn,
                    None,
                )?
            }
            SampleFormat::I32 => {
                let buffer = Arc::clone(&self.buffer);
                self.device.build_input_stream(
                    &self.config,
                    capture(buffer, channels, device_rate, |s: i32| {
                        s as f32 / i32::MAX as f32
                    }),
                    err_fn,
                    None,
                )?
            }
            SampleFormat::F32 => {
                let buffer = Arc::clone(&self.buffer);
                self.device.build_input_stream(
                    &self.config,
                    capture(buffer, channels, device_rate, |s: f32| s),
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

        let at = lock_recover(&self.buffer).len_abs();

        crate::log_debug!(
            "VoiceRecorder",
            format!(
                "録音停止: at={}, 累計={:.1}秒, デバイスレート={}",
                at,
                at as f32 / SAMPLE_RATE as f32,
                self.device_sample_rate
            )
        );
    }

    /// 現在の絶対サンプル位置（失われた分・過去の録音分を含む累計）。
    ///
    /// 録音を取り直しても巻き戻らないため、位置は録音をまたいで一意になる。
    pub fn buffer_len(&self) -> usize {
        lock_recover(&self.buffer).len_abs()
    }

    /// 指定位置（絶対インデックス）以降のサンプルを取得（録音を停止せずに）。
    ///
    /// サンプルも位置も `SAMPLE_RATE` 基準。デバイスのレートは録音時に吸収済み。
    ///
    /// 保持上限を超えて古い側が失われている場合、`RecordedSlice::start` が
    /// 要求位置より後ろになる。
    pub fn get_samples_since(&self, start_sample: usize) -> RecordedSlice {
        lock_recover(&self.buffer).samples_since(start_sample)
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

    /// i16 で正確に往復する値を作る。サンプル値を識別子代わりに使うため。
    fn s(n: i16) -> f32 {
        n as f32 / i16::MAX as f32
    }

    #[test]
    fn buffer_keeps_absolute_indices_across_overwrite() {
        let mut buf = RecordingBuffer::with_capacity(6);
        for i in 0..10i16 {
            buf.push(s(i));
        }
        assert_eq!(buf.len_abs(), 10);

        // 先頭が失われても絶対インデックスは不変
        assert_eq!(buf.samples.len(), 6); // 実メモリは上限に収まる
        let slice = buf.samples_since(4);
        assert_eq!(slice.samples, vec![s(4), s(5), s(6), s(7), s(8), s(9)]);

        // さらに録音が進んでも整合する
        buf.push(s(10));
        let slice2 = buf.samples_since(8);
        assert_eq!(slice2.samples, vec![s(8), s(9), s(10)]);
    }

    #[test]
    fn buffer_round_trips_samples_and_clamps_out_of_range() {
        let mut buf = RecordingBuffer::default();
        for s in [0.0, 0.5, -0.5, 1.0, -1.0, 2.0, -2.0] {
            buf.push(s);
        }

        let got = buf.samples_since(0).samples;
        // 範囲内はほぼそのまま、範囲外は ±1.0 に丸められる
        for (got, want) in got.iter().zip([0.0, 0.5, -0.5, 1.0, -1.0, 1.0, -1.0]) {
            assert!((got - want).abs() < 1e-3, "got={got}, want={want}");
        }
    }

    #[test]
    fn buffer_drops_oldest_samples_beyond_capacity() {
        let mut buf = RecordingBuffer::with_capacity(4);
        for i in 0..6i16 {
            buf.push(s(i));
        }

        // 絶対インデックスは録音開始からの通し番号のまま
        assert_eq!(buf.len_abs(), 6);
        // 実メモリは上限に収まり、新しい側が残る
        assert_eq!(buf.samples.len(), 4);
        let slice = buf.samples_since(2);
        assert_eq!(slice.samples, vec![s(2), s(3), s(4), s(5)]);
    }

    #[test]
    fn buffer_reports_actual_start_when_requested_position_was_dropped() {
        let mut buf = RecordingBuffer::with_capacity(4);
        for i in 0..6i16 {
            buf.push(s(i));
        }

        // 絶対位置 0,1 は容量超過で失われている
        let slice = buf.samples_since(0);

        assert_eq!(slice.start, 2, "失われた分だけ開始位置が後ろへずれる");
        assert_eq!(slice.samples, vec![s(2), s(3), s(4), s(5)]);
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

    #[test]
    fn clearing_the_buffer_does_not_rewind_absolute_indices() {
        let mut buf = RecordingBuffer::with_capacity(6);
        for i in 0..10i16 {
            buf.push(s(i));
        }

        // 録音を取り直しても、以前の位置を新しい録音が名乗り直すことはない
        buf.clear();

        assert_eq!(buf.len_abs(), 10);
        buf.push(s(99));
        assert_eq!(buf.samples_since(10).start, 10);
    }
}
