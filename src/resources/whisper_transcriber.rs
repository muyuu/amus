//! Whisper音声書き起こしリソース
//!
//! whisper.cpp を使用して音声データをテキストに変換する。

use thiserror::Error;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Whisper による書き起こしが失敗した理由。
#[derive(Debug, Error)]
pub enum TranscribeError {
    #[error("Whisper モデルの読み込みに失敗: {0}")]
    LoadModel(whisper_rs::WhisperError),
    #[error("Whisper 状態の作成に失敗: {0}")]
    CreateState(whisper_rs::WhisperError),
    #[error("書き起こしに失敗: {0}")]
    Run(whisper_rs::WhisperError),
}

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
        .join("ggml-small.bin")
        .to_string_lossy()
        .to_string()
}

/// モデルが存在するかチェック
pub fn model_exists() -> bool {
    std::path::Path::new(&get_model_path()).exists()
}

/// モデルのダウンロードURL
pub fn get_model_download_url() -> &'static str {
    // Whisper small model (約466MB) - 精度重視
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
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
    pub fn new(model_path: &str) -> Result<Self, TranscribeError> {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(TranscribeError::LoadModel)?;

        Ok(Self { ctx })
    }

    /// 音声データを書き起こし
    /// samples: 16kHz, mono, f32の音声データ
    /// context: 認識精度向上のためのコンテキスト（プレイヤー名など）
    /// 戻り値: タイムスタンプ付きのテキストセグメント
    pub fn transcribe(
        &self,
        samples: &[f32],
        context: Option<&str>,
    ) -> Result<Vec<TranscriptionSegment>, TranscribeError> {
        // 短すぎるサンプルはスキップ（0.5秒未満）
        if samples.len() < 8000 {
            return Ok(Vec::new());
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
        params.set_single_segment(false); // 複数セグメント許可

        if let Some(ctx) = context {
            params.set_initial_prompt(ctx);
        }

        // 書き起こし実行
        let mut state = self
            .ctx
            .create_state()
            .map_err(TranscribeError::CreateState)?;

        state.full(params, samples).map_err(TranscribeError::Run)?;

        // テキストを取得
        let mut segments = Vec::new();

        for segment in state.as_iter() {
            if let Ok(text) = segment.to_str_lossy() {
                // 文字化け（置換文字）を除去
                let clean_text: String = text.chars().filter(|c| *c != '\u{FFFD}').collect();
                let clean_text = remove_sound_effects(&clean_text);
                let clean_text = clean_text.trim();

                if !clean_text.is_empty() {
                    // セグメントの開始時刻（ミリ秒）
                    let start_secs = segment.start_timestamp() as f32 / 1000.0;

                    segments.push(TranscriptionSegment {
                        timestamp_secs: start_secs,
                        text: clean_text.to_string(),
                    });
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
