//! 音声録音リソース
//!
//! マイクからの音声入力を録音し、16kHz mono f32 形式で出力する。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use thiserror::Error;

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
    buffer: Arc<Mutex<Vec<f32>>>,
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

        eprintln!(
            "VoiceRecorder: sample_format={:?}, sample_rate={}, channels={}",
            sample_format, sample_rate, config.channels
        );

        Ok(Self {
            device,
            config,
            sample_format,
            sample_rate,
            buffer: Arc::new(Mutex::new(Vec::new())),
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
        let err_fn = |err| eprintln!("録音エラー: {}", err);

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

        let samples = {
            let buffer = lock_recover(&self.buffer);
            buffer.clone()
        };

        let duration_secs = {
            let start_time = lock_recover(&self.start_time);
            start_time
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(0.0)
        };

        eprintln!(
            "録音停止: {}サンプル取得, 録音時間={:.1}秒, 元サンプルレート={}",
            samples.len(),
            duration_secs,
            self.sample_rate
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

    /// 現在のサンプルバッファのサイズ（サンプル数）
    pub fn buffer_len(&self) -> usize {
        lock_recover(&self.buffer).len()
    }

    /// 元のサンプルレートを取得
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 指定位置以降のサンプルを取得（録音を停止せずに）
    /// 逐次書き起こし用。戻り値は (16kHzリサンプリング済みサンプル, 次回開始位置)
    pub fn get_samples_since(&self, start_sample: usize) -> (Vec<f32>, usize) {
        let buffer = lock_recover(&self.buffer);
        if start_sample >= buffer.len() {
            return (Vec::new(), start_sample);
        }

        let samples = buffer[start_sample..].to_vec();
        let next_position = buffer.len();
        drop(buffer); // ロックを解放

        // 16kHzにリサンプリング
        let resampled = self.resample_to_16k(&samples);
        (resampled, next_position)
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
