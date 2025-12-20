use super::VoiceMemo;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

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

    /// 音声データを書き起こし
    /// samples: 16kHz, mono, f32の音声データ
    /// context: 認識精度向上のためのコンテキスト（プレイヤー名など）
    /// 戻り値: タイムスタンプ付きのテキストセグメント
    pub fn transcribe(
        &self,
        samples: &[f32],
        context: Option<&str>,
    ) -> Result<Vec<VoiceMemo>, String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // 日本語に設定
        params.set_language(Some("ja"));

        // タイムスタンプを有効化
        params.set_token_timestamps(true);

        // 処理を高速化するための設定
        params.set_n_threads(4);

        // 短い音声でも処理できるように
        params.set_no_context(true);

        // 単一セグメントにまとめない
        params.set_single_segment(false);

        // 初期プロンプト（コンテキスト）を設定
        if let Some(ctx) = context {
            params.set_initial_prompt(ctx);
        }

        // 書き起こし実行
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("Whisper状態の作成に失敗: {}", e))?;

        state
            .full(params, samples)
            .map_err(|e| format!("書き起こしに失敗: {}", e))?;

        // トークンレベルでタイムスタンプを取得し、発話ごとに分割
        let num_segments = state.full_n_segments().map_err(|e| format!("セグメント数の取得に失敗: {}", e))?;
        let mut memos = Vec::new();

        // 発話を検出するための閾値（秒）- この間隔以上空いたら別の発話とみなす
        const PAUSE_THRESHOLD_SECS: f32 = 0.8;

        let mut current_text = String::new();
        let mut current_start: Option<f32> = None;
        let mut last_end: f32 = 0.0;

        for seg_idx in 0..num_segments {
            let num_tokens = state
                .full_n_tokens(seg_idx)
                .map_err(|e| format!("トークン数の取得に失敗: {}", e))?;

            for tok_idx in 0..num_tokens {
                let token_data = match state.full_get_token_data(seg_idx, tok_idx) {
                    Ok(data) => data,
                    Err(_) => continue, // データ取得失敗はスキップ
                };

                let token_text = match state.full_get_token_text(seg_idx, tok_idx) {
                    Ok(text) => text,
                    Err(_) => continue, // UTF-8エラー等はスキップ
                };

                // 特殊トークンをスキップ
                if token_text.is_empty()
                    || token_text.starts_with('[')
                    || token_text.starts_with('<')
                    || token_data.id >= 50257  // 特殊トークンID
                {
                    continue;
                }

                // 句読点のみのトークンかどうか
                let is_punctuation = token_text.chars().all(|c| {
                    matches!(c, '、' | '。' | ',' | '.' | '!' | '?' | '！' | '？' | '…')
                });

                let t0_secs = token_data.t0 as f32 / 100.0;
                let t1_secs = token_data.t1 as f32 / 100.0;

                // 間隔が閾値以上空いたら、前の発話を保存して新しい発話を開始
                if current_start.is_some() && (t0_secs - last_end) > PAUSE_THRESHOLD_SECS {
                    let text = current_text.trim();
                    if !text.is_empty() {
                        memos.push(VoiceMemo {
                            timestamp_secs: current_start.unwrap(),
                            text: text.to_string(),
                        });
                    }
                    current_text.clear();
                    current_start = None;
                }

                // 発話の開始
                if current_start.is_none() {
                    current_start = Some(t0_secs);
                }

                current_text.push_str(&token_text);
                // 句読点以外の場合のみ last_end を更新（句読点は間隔計算に含めない）
                if !is_punctuation {
                    last_end = t1_secs;
                }
            }
        }

        // 最後の発話を保存
        let text = current_text.trim();
        if !text.is_empty() {
            if let Some(start) = current_start {
                memos.push(VoiceMemo {
                    timestamp_secs: start,
                    text: text.to_string(),
                });
            }
        }

        // タイムスタンプ補正: 音声の最初の有音部分を検出してオフセットとして加算
        let first_speech_offset = detect_first_speech(samples);
        for memo in &mut memos {
            memo.timestamp_secs += first_speech_offset;
        }

        Ok(memos)
    }
}

/// 音声データの最初の有音部分を検出（秒数を返す）
fn detect_first_speech(samples: &[f32]) -> f32 {
    const SAMPLE_RATE: f32 = 16000.0;
    const WINDOW_SIZE: usize = 1600; // 100ms window
    const THRESHOLD: f32 = 0.01; // RMSエネルギー閾値

    for (i, window) in samples.chunks(WINDOW_SIZE).enumerate() {
        // RMSエネルギーを計算
        let rms: f32 = (window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32).sqrt();

        if rms > THRESHOLD {
            return (i * WINDOW_SIZE) as f32 / SAMPLE_RATE;
        }
    }

    0.0 // 有音が見つからなければ0を返す
}

/// Whisperモデルのダウンロード先を取得
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
    // Whisper small model (約466MB) - より高精度
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
}

/// モデルサイズの説明
pub fn get_model_size_description() -> &'static str {
    "約466MB"
}

/// モデルのダウンロード進捗
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

impl DownloadProgress {
    pub fn percentage(&self) -> Option<f32> {
        self.total_bytes.map(|total| {
            if total == 0 {
                0.0
            } else {
                (self.downloaded_bytes as f32 / total as f32) * 100.0
            }
        })
    }
}

/// モデルをダウンロード（バックグラウンドスレッド用）
/// progress_tx: 進捗を送信するチャネル
pub fn download_model(
    progress_tx: std::sync::mpsc::Sender<DownloadProgress>,
) -> Result<(), String> {
    use std::io::{Read, Write};

    let url = get_model_download_url();
    let model_path = get_model_path();

    // ディレクトリを作成
    if let Some(parent) = std::path::Path::new(&model_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("ディレクトリ作成に失敗: {}", e))?;
    }

    // ダウンロード開始
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("ダウンロード開始に失敗: {}", e))?;

    let total_bytes = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());

    // 一時ファイルに書き込み
    let temp_path = format!("{}.download", model_path);
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("ファイル作成に失敗: {}", e))?;

    let mut reader = response.into_reader();
    let mut buffer = [0u8; 8192];
    let mut downloaded_bytes = 0u64;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("読み込みエラー: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("書き込みエラー: {}", e))?;

        downloaded_bytes += bytes_read as u64;

        // 進捗を送信（エラーは無視、受信側が閉じている場合）
        let _ = progress_tx.send(DownloadProgress {
            downloaded_bytes,
            total_bytes,
        });
    }

    // 一時ファイルをリネーム
    std::fs::rename(&temp_path, &model_path)
        .map_err(|e| format!("ファイル移動に失敗: {}", e))?;

    Ok(())
}
