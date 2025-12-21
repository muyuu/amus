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
        const SAMPLE_RATE: usize = 16000;

        // 音声データから発話区間を検出
        let speech_segments = detect_speech_segments(samples);
        eprintln!("検出された発話区間: {:?}", speech_segments);

        let mut memos = Vec::new();

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
            params.set_n_threads(4);
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
                eprintln!("区間 {:.1}s-{:.1}s の書き起こしに失敗: {}", start_secs, end_secs, e);
                continue;
            }

            // テキストを取得
            let mut text = String::new();
            for segment in state.as_iter() {
                if let Ok(seg_text) = segment.to_str_lossy() {
                    // 文字化け（置換文字）を除去
                    let clean_text: String = seg_text
                        .chars()
                        .filter(|c| *c != '\u{FFFD}')
                        .collect();
                    text.push_str(&clean_text);
                }
            }

            let text = remove_sound_effects(&text);
            let text = text.trim();

            if !text.is_empty() {
                // 「。」で分割して複数のメモにする
                let sentences: Vec<&str> = text.split('。').collect();
                for (i, sentence) in sentences.iter().enumerate() {
                    let sentence = sentence.trim();
                    if !sentence.is_empty() {
                        eprintln!("  {:.1}s: {:?}", start_secs, sentence);
                        memos.push(VoiceMemo {
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

        Ok(memos)
    }
}

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
                    let speech_end = current_time - (MIN_SILENCE_WINDOWS as f32 * WINDOW_SIZE as f32 / SAMPLE_RATE);
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
        .headers()
        .get("Content-Length")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // 一時ファイルに書き込み
    let temp_path = format!("{}.download", model_path);
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("ファイル作成に失敗: {}", e))?;

    let mut reader = response.into_body().into_reader();
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
