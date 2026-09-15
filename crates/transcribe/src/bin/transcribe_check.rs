//! 音声ファイルを指定モデルで書き起こして表示する検証用CLI。
//!
//! アプリ本体を再ビルドせずに、モデルや認識パラメータ（プロンプト等）を試せるように
//! するためのもの。対応する音声形式は WAV（16bit/24bit/32bit PCM・float、任意サンプル
//! レート・チャンネル数）。`record-audio` feature 有効時は MP3 も読める（`record-audio`
//! ビルドが集めたデータをそのまま渡せるようにするため）。
//!
//! 使い方:
//!   transcribe-check --model small|medium [--gpu vulkan|cuda|metal] \
//!       [--prompt "テキスト"] <音声ファイル.wav|.mp3>...
//!
//! `--model-path <ファイル>` で、アプリのレジストリに無い任意の GGML ファイルを
//! 直接指定できる（ダウンロード・SHA-256検証はしない、既に手元にある前提）。
//! `--model` と併用した場合は `--model-path` が優先される。

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
    let mut model_path: Option<String> = None;
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
                    "medium" => WhisperModel::MEDIUM,
                    other => {
                        eprintln!("未知のモデル: {other}（small | medium）");
                        return ExitCode::FAILURE;
                    }
                };
                i += 2;
            }
            "--model-path" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--model-path には値が要ります");
                    return ExitCode::FAILURE;
                };
                model_path = Some(value.clone());
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
            "使い方: transcribe-check --model small|medium | --model-path <ファイル> \\\n    [--gpu vulkan|cuda|metal] [--prompt \"テキスト\"] <音声ファイル.wav|.mp3>..."
        );
        return ExitCode::FAILURE;
    }

    let model = if let Some(path) = model_path {
        let path: &'static str = Box::leak(path.into_boxed_str());
        let model = WhisperModel::at_path(path);
        if !model.exists() {
            eprintln!("指定されたモデルファイルが見つかりません: {path}");
            return ExitCode::FAILURE;
        }
        model
    } else {
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
        model
    };

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
        let samples = match load_audio_as_16k_mono(path) {
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

/// 拡張子で WAV / MP3（`record-audio` feature 時のみ）を振り分けて読む。
fn load_audio_as_16k_mono(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let is_mp3 = path
        .rsplit('.')
        .next()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"));

    if is_mp3 {
        #[cfg(feature = "record-audio")]
        return load_mp3_as_16k_mono(path);
        #[cfg(not(feature = "record-audio"))]
        return Err("MP3 を読むには --features record-audio でビルドしてください".into());
    }

    load_wav_as_16k_mono(path)
}

/// チャンネル平均でモノラル化し、16kHz へリサンプルする（WAV/MP3 共通）。
fn to_16k_mono(raw: Vec<f32>, channels: usize, sample_rate: u32) -> Vec<f32> {
    use transcribe::resampler::Resampler;
    use transcribe::voice_recorder::SAMPLE_RATE;

    let mono: Vec<f32> = if channels <= 1 {
        raw
    } else {
        raw.chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
            .collect()
    };

    if sample_rate == SAMPLE_RATE {
        return mono;
    }
    let mut resampler = Resampler::new(sample_rate, SAMPLE_RATE);
    let mut out = Vec::new();
    resampler.process(&mono, |s| out.push(s));
    out
}

/// WAV を読み、Whisper が要求する 16kHz mono f32 へ変換する。
fn load_wav_as_16k_mono(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
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

    Ok(to_16k_mono(raw, spec.channels as usize, spec.sample_rate))
}

/// MP3（`record-audio` が集めたデータ）を読み、16kHz mono f32 へ変換する。
#[cfg(feature = "record-audio")]
fn load_mp3_as_16k_mono(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error as SymphoniaError;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("MP3 に音声トラックが見つかりません")?;
    let track_id = track.id;
    let channels = track
        .codec_params
        .channels
        .ok_or("MP3 のチャンネル数が不明です")?
        .count();
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or("MP3 のサンプルレートが不明です")?;

    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut raw: Vec<f32> = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(e) => return Err(e.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = decoder.decode(&packet)?;
        let spec = *decoded.spec();
        let mut sample_buf =
            symphonia::core::audio::SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        raw.extend_from_slice(sample_buf.samples());
    }

    Ok(to_16k_mono(raw, channels, sample_rate))
}
