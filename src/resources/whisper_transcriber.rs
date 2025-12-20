//! Whisper音声書き起こしリソース
//!
//! whisper.cpp を使用して音声データをテキストに変換する。

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// 書き起こし結果のセグメント
#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    /// 録音開始からの秒数
    pub timestamp_secs: f32,
    /// 書き起こしテキスト
    pub text: String,
}

// =============================================================================
// Whisperモデル設定
// =============================================================================

/// Whisperモデルの保存先パスを取得
pub fn get_model_path() -> String {
    // アプリのデータディレクトリにモデルを配置
    let data_dir = dirs_next::data_dir()
        .map(|p| p.join("amus"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    data_dir
        .join("models")
        .join("ggml-base.bin")
        .to_string_lossy()
        .to_string()
}

/// モデルが存在するかチェック
pub fn model_exists() -> bool {
    std::path::Path::new(&get_model_path()).exists()
}

/// モデルのダウンロードURL
pub fn get_model_download_url() -> &'static str {
    // Whisper base model (約142MB) - 速度と精度のバランス
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
}

/// モデルサイズの説明
pub fn get_model_size_description() -> &'static str {
    "約142MB"
}

// =============================================================================
// WhisperTranscriber
// =============================================================================

/// Whisperによる音声書き起こし
pub struct WhisperTranscriber {
    ctx: WhisperContext,
}

impl WhisperTranscriber {
    /// 新しいWhisperTranscriberを作成
    /// model_path: Whisperモデルファイルへのパス（.bin）
    pub fn new(model_path: &str) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| format!("Whisperモデルの読み込みに失敗: {}", e))?;

        Ok(Self { ctx })
    }

    /// デフォルトのモデルパスで初期化
    pub fn with_default_model() -> Result<Self, String> {
        Self::new(&get_model_path())
    }

    /// 音声データを書き起こし
    /// samples: 16kHz, mono, f32の音声データ
    /// context: 認識精度向上のためのコンテキスト（プレイヤー名など）
    /// 戻り値: タイムスタンプ付きのテキストセグメント
    pub fn transcribe(
        &self,
        samples: &[f32],
        context: Option<&str>,
    ) -> Result<Vec<TranscriptionSegment>, String> {
        const SAMPLE_RATE: usize = 16000;

        // 音声データから発話区間を検出
        let speech_segments = detect_speech_segments(samples);
        eprintln!("検出された発話区間: {:?}", speech_segments);

        let mut segments = Vec::new();

        // 各発話区間ごとにWhisperを実行
        for (start_secs, end_secs) in speech_segments {
            let start_sample = (start_secs * SAMPLE_RATE as f32) as usize;
            let end_sample = (end_secs * SAMPLE_RATE as f32) as usize;

            if end_sample <= start_sample || end_sample > samples.len() {
                continue;
            }

            let segment_samples = &samples[start_sample..end_sample];

            // 短すぎる区間はスキップ（0.5秒未満）
            if segment_samples.len() < SAMPLE_RATE / 2 {
                continue;
            }

            // Whisperパラメータ設定
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(Some("ja"));
            // CPUコア数の75%を使用（最低4、最大12）
            let n_threads = std::thread::available_parallelism()
                .map(|n| (n.get() * 3 / 4).clamp(4, 12) as i32)
                .unwrap_or(4);
            params.set_n_threads(n_threads);
            params.set_no_context(true);
            params.set_single_segment(true); // 短い区間なので単一セグメント

            if let Some(ctx) = context {
                params.set_initial_prompt(ctx);
            }

            // 書き起こし実行
            let mut state = self
                .ctx
                .create_state()
                .map_err(|e| format!("Whisper状態の作成に失敗: {}", e))?;

            if let Err(e) = state.full(params, segment_samples) {
                eprintln!(
                    "区間 {:.1}s-{:.1}s の書き起こしに失敗: {}",
                    start_secs, end_secs, e
                );
                continue;
            }

            // テキストを取得
            let mut text = String::new();
            for segment in state.as_iter() {
                if let Ok(seg_text) = segment.to_str_lossy() {
                    // 文字化け（置換文字）を除去
                    let clean_text: String =
                        seg_text.chars().filter(|c| *c != '\u{FFFD}').collect();
                    text.push_str(&clean_text);
                }
            }

            let text = remove_sound_effects(&text);
            let text = text.trim();

            if !text.is_empty() {
                // 「。」で分割して複数のセグメントにする
                let sentences: Vec<&str> = text.split('。').collect();
                for (i, sentence) in sentences.iter().enumerate() {
                    let sentence = sentence.trim();
                    if !sentence.is_empty() {
                        eprintln!("  {:.1}s: {:?}", start_secs, sentence);
                        segments.push(TranscriptionSegment {
                            timestamp_secs: start_secs,
                            text: if i < sentences.len() - 1 || text.ends_with('。') {
                                format!("{}。", sentence)
                            } else {
                                sentence.to_string()
                            },
                        });
                    }
                }
            }
        }

        Ok(segments)
    }
}

// =============================================================================
// ヘルパー関数
// =============================================================================

/// 効果音（カッコ付きテキスト）を除去
fn remove_sound_effects(text: &str) -> String {
    let mut result = text.to_string();

    // [xxx] 形式を除去
    while let Some(start) = result.find('[') {
        if let Some(end) = result[start..].find(']') {
            result.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }

    // (xxx) 形式を除去
    while let Some(start) = result.find('(') {
        if let Some(end) = result[start..].find(')') {
            result.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }

    // （xxx） 形式（全角）を除去
    while let Some(start) = result.find('（') {
        if let Some(end) = result[start..].find('）') {
            result.replace_range(start..start + end + '）'.len_utf8(), "");
        } else {
            break;
        }
    }

    result.trim().to_string()
}

/// 音声データから発話区間を検出
/// 戻り値: (開始秒, 終了秒) のリスト
fn detect_speech_segments(samples: &[f32]) -> Vec<(f32, f32)> {
    const SAMPLE_RATE: f32 = 16000.0;
    const WINDOW_SIZE: usize = 1600; // 100ms window
    const THRESHOLD: f32 = 0.01; // RMSエネルギー閾値
    const MIN_SILENCE_WINDOWS: usize = 8; // 0.8秒以上の無音で分割

    let mut segments = Vec::new();
    let mut in_speech = false;
    let mut speech_start = 0.0f32;
    let mut silence_count = 0usize;

    for (i, window) in samples.chunks(WINDOW_SIZE).enumerate() {
        let rms: f32 = (window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32).sqrt();
        let current_time = (i * WINDOW_SIZE) as f32 / SAMPLE_RATE;

        if rms > THRESHOLD {
            // 有音
            if !in_speech {
                speech_start = current_time;
                in_speech = true;
            }
            silence_count = 0;
        } else {
            // 無音
            if in_speech {
                silence_count += 1;
                if silence_count >= MIN_SILENCE_WINDOWS {
                    // 十分な無音があったので発話区間を終了
                    let speech_end = current_time
                        - (MIN_SILENCE_WINDOWS as f32 * WINDOW_SIZE as f32 / SAMPLE_RATE);
                    segments.push((speech_start, speech_end));
                    in_speech = false;
                    silence_count = 0;
                }
            }
        }
    }

    // 最後の発話区間
    if in_speech {
        let end_time = samples.len() as f32 / SAMPLE_RATE;
        segments.push((speech_start, end_time));
    }

    segments
}
