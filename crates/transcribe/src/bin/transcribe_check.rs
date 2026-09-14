//! 音声ファイルを指定モデルで書き起こして表示する検証用CLI。
//!
//! アプリ本体を再ビルドせずに、モデルや認識パラメータ（プロンプト等）を試せるように
//! するためのもの。対応する音声形式は WAV のみ（16bit/24bit/32bit PCM・float、
//! 任意サンプルレート・チャンネル数）。MP3 は `cargo test` 側の
//! `whisper_transcriber::recognition_check` を使うこと。
//!
//! 使い方:
//!   transcribe-check --model small|kotoba [--gpu vulkan|cuda|metal] \
//!       [--prompt "テキスト"] <音声ファイル.wav>...

use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

use transcribe::whisper_backend::{GpuBackend, TranscribeSetup};
use transcribe::whisper_transcriber::WhisperModel;
use transcribe::{IntegrityCheck, WhisperTranscriber};

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut model = WhisperModel::SMALL;
    let mut gpu = None;
    let mut prompt: Option<String> = None;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--model には値が要ります");
                    return ExitCode::FAILURE;
                };
                model = match value.as_str() {
                    "small" => WhisperModel::SMALL,
                    "kotoba" => WhisperModel::KOTOBA_V2,
                    other => {
                        eprintln!("未知のモデル: {other}（small | kotoba）");
                        return ExitCode::FAILURE;
                    }
                };
                i += 2;
            }
            "--gpu" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--gpu には値が要ります");
                    return ExitCode::FAILURE;
                };
                gpu = Some(match value.as_str() {
                    "vulkan" => GpuBackend::Vulkan,
                    "cuda" => GpuBackend::Cuda,
                    "metal" => GpuBackend::Metal,
                    other => {
                        eprintln!("未知のGPUバックエンド: {other}（vulkan | cuda | metal）");
                        return ExitCode::FAILURE;
                    }
                });
                i += 2;
            }
            "--prompt" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--prompt には値が要ります");
                    return ExitCode::FAILURE;
                };
                prompt = Some(value.clone());
                i += 2;
            }
            path => {
                files.push(path.to_string());
                i += 1;
            }
        }
    }

    if files.is_empty() {
        eprintln!(
            "使い方: transcribe-check --model small|kotoba [--gpu vulkan|cuda|metal] \\\n    [--prompt \"テキスト\"] <音声ファイル.wav>..."
        );
        return ExitCode::FAILURE;
    }

    if !model.exists() {
        println!(
            "モデル {} が未取得のためダウンロードします（{}）...",
            model.name(),
            model.size_label()
        );
        if let Err(e) = download_model(model) {
            eprintln!("ダウンロードに失敗: {e}");
            return ExitCode::FAILURE;
        }
    }

    let setup = TranscribeSetup { gpu, model };
    let mut transcriber = match WhisperTranscriber::new(setup) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("モデルの読み込みに失敗: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("構成: {}\n", transcriber.setup().label());

    for path in &files {
        let samples = match load_wav_as_16k_mono(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[{path}] 読み込みに失敗: {e}");
                continue;
            }
        };

        match transcriber.transcribe(&samples, prompt.as_deref()) {
            Ok(segments) => {
                println!("[{path}]");
                if segments.is_empty() {
                    println!("  (0セグメント)");
                }
                for seg in segments {
                    println!(
                        "  {:>6.1}-{:>6.1}s: {}",
                        seg.start_secs, seg.end_secs, seg.text
                    );
                }
            }
            Err(e) => eprintln!("[{path}] 書き起こしに失敗: {e}"),
        }
    }

    ExitCode::SUCCESS
}

fn download_model(model: WhisperModel) -> Result<(), transcribe::DownloadError> {
    let (tx, rx) = mpsc::channel::<transcribe::DownloadProgress>();
    let cancel = Arc::new(AtomicBool::new(false));
    let handle = std::thread::spawn(move || {
        for progress in rx {
            if let Some(pct) = progress.percentage() {
                print!("\r  {pct:.0}%");
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
        }
        println!();
    });

    let check = IntegrityCheck {
        max_bytes: Some(model.max_download_bytes()),
        sha256_hex: Some(model.sha256()),
    };
    let result = transcribe::download_file(model.url(), &model.path(), tx, cancel, &check);
    let _ = handle.join();
    result
}

/// WAV を読み、Whisper が要求する 16kHz mono f32 へ変換する。
fn load_wav_as_16k_mono(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    use transcribe::resampler::Resampler;
    use transcribe::voice_recorder::SAMPLE_RATE;

    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    let raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let max_val = (1i64 << (spec.bits_per_sample.max(1) - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max_val))
                .collect::<Result<_, _>>()?
        }
    };

    // ステレオ等は全チャンネル平均でモノラル化する（voice_recorder と同じ方針）。
    let channels = spec.channels as usize;
    let mono: Vec<f32> = if channels <= 1 {
        raw
    } else {
        raw.chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
            .collect()
    };

    if spec.sample_rate == SAMPLE_RATE {
        return Ok(mono);
    }
    let mut resampler = Resampler::new(spec.sample_rate, SAMPLE_RATE);
    let mut out = Vec::new();
    resampler.process(&mono, |s| out.push(s));
    Ok(out)
}
