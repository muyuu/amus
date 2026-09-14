//! Whisper音声書き起こしリソース
//!
//! whisper.cpp を使用して音声データをテキストに変換する。

use super::whisper_backend::TranscribeSetup;
use thiserror::Error;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

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

/// 書き起こされたトークン1つ。
///
/// 時刻は渡したサンプル列の先頭を 0 とする相対秒。
struct SpokenToken {
    start_secs: f32,
    end_secs: f32,
    text: String,
}

/// 書き起こし結果のセグメント。
///
/// 時刻は渡したサンプル列の先頭を 0 とする相対秒であり、録音上の位置ではない。
#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    /// セグメント開始（秒）
    pub start_secs: f32,
    /// セグメント終了（秒）
    pub end_secs: f32,
    /// 書き起こしテキスト
    pub text: String,
}

// =============================================================================
// Whisperモデル設定
// =============================================================================

/// ダウンロードして使う Whisper モデル。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhisperModel {
    file_name: &'static str,
    /// 上流の差し替えに追従しないよう `main` ではなく revision に固定する。
    url: &'static str,
    /// DL 完了後・配置前に照合する SHA-256（16進小文字）。不一致なら破棄する。
    sha256: &'static str,
    expected_bytes: u64,
}

impl WhisperModel {
    /// 約466MB。
    pub const SMALL: Self = Self {
        file_name: "ggml-small.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-small.bin",
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        expected_bytes: 487_601_967,
    };

    /// 約1.5GB。GPU バックエンドを含むビルドで使う。
    pub const MEDIUM: Self = Self {
        file_name: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-medium.bin",
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        expected_bytes: 1_533_763_059,
    };

    /// モデルの通称（`small` / `medium`）。ログや画面に出す。
    pub fn name(&self) -> &'static str {
        self.file_name
            .trim_start_matches("ggml-")
            .trim_end_matches(".bin")
    }

    /// ダウンロード量の目安表記。`約 466 MB` / `約 1.4 GB` のような形。
    pub fn size_label(&self) -> String {
        const MB: f64 = 1024.0 * 1024.0;
        let mb = self.expected_bytes as f64 / MB;

        if mb >= 1024.0 {
            format!("約 {:.1} GB", mb / 1024.0)
        } else {
            format!("約 {:.0} MB", mb)
        }
    }

    /// モデルファイルの保存先。
    pub fn path(&self) -> String {
        let data_dir = dirs_next::data_dir()
            .map(|p| p.join("amus"))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        data_dir
            .join("models")
            .join(self.file_name)
            .to_string_lossy()
            .to_string()
    }

    pub fn exists(&self) -> bool {
        std::path::Path::new(&self.path()).exists()
    }

    pub fn url(&self) -> &'static str {
        self.url
    }

    pub fn sha256(&self) -> &'static str {
        self.sha256
    }

    /// 書き込みを打ち切る上限バイト数。
    /// Content-Length を信頼せず、暴走ダウンロードを防ぐための上限。
    pub fn max_download_bytes(&self) -> u64 {
        self.expected_bytes + 4 * 1024 * 1024
    }

    /// レジストリ外の任意の GGML ファイルを直接指定する（検証用）。
    ///
    /// ダウンロード・SHA-256 検証の対象外。`path` は絶対パスであること
    /// （[`Self::path`] の実装上、絶対パスならそのまま使われる）。
    pub fn at_path(path: &'static str) -> Self {
        Self {
            file_name: path,
            url: "",
            sha256: "",
            expected_bytes: 0,
        }
    }
}

// =============================================================================
// WhisperTranscriber
// =============================================================================

/// Whisperによる音声書き起こし
pub struct WhisperTranscriber {
    ctx: WhisperContext,
    /// どの構成で動いているか。要求した構成とは限らず、GPU の初期化に失敗していれば
    /// CPU に落ちたあとの実態を指す。
    setup: TranscribeSetup,
    /// 推論状態。確保が重い（数百MB）ので使い回す。
    ///
    /// `whisper_full` は呼び出しの先頭で結果を破棄するため、跨いで持ち越すものはない。
    /// トークナイズにしか使わないインスタンスもあるため、最初の書き起こしまで作らない。
    state: Option<WhisperState>,
}

impl WhisperTranscriber {
    /// 指定の構成で書き起こしを用意する。
    ///
    /// GPU を含む構成で GPU が使えなかった場合は、CPU の構成で作り直して返す。
    /// 実際に動いている構成は [`Self::setup`] で分かる。モデルファイルが無い、
    /// あるいは CPU でも読めなかった場合だけ失敗する。
    pub fn new(setup: TranscribeSetup) -> Result<Self, TranscribeError> {
        match Self::open(setup) {
            Ok(transcriber) => Ok(transcriber),
            Err(e) if setup.gpu.is_some() => {
                crate::log_error!(
                    "WhisperTranscriber",
                    format!("GPU での初期化に失敗したため CPU で続行する: {}", e)
                );
                Self::open(TranscribeSetup::CPU)
            }
            Err(e) => Err(e),
        }
    }

    /// 構成どおりに開く。退避はしない。
    fn open(setup: TranscribeSetup) -> Result<Self, TranscribeError> {
        // whisper.cpp / GGML は既定で stderr へ直接大量に出力し、こちらのログを埋める。
        // `log` へ寄せることでフィルタの対象になり、既定（自クレート以外は Off）では
        // 出なくなる。複数回呼んでも安全。
        whisper_rs::install_logging_hooks();

        let mut params = WhisperContextParameters::default();
        // 既定は「ビルドに GPU バックエンドがあれば使う」。選択を反映するため明示する。
        params.use_gpu(setup.gpu.is_some());

        let ctx = WhisperContext::new_with_params(setup.model.path(), params)
            .map_err(TranscribeError::LoadModel)?;

        Ok(Self {
            ctx,
            setup,
            state: None,
        })
    }

    /// 実際に動いている構成。
    pub fn setup(&self) -> TranscribeSetup {
        self.setup
    }

    /// テキストがモデルのトークナイザで何トークンになるかを返す。
    ///
    /// 認識コンテキストを `initial_prompt` の上限に収めるために使う。
    /// トークン分割はモデルごとに異なるため、文字数からは推定できない。
    pub fn count_tokens(&self, text: &str) -> usize {
        self.ctx
            .tokenize(text, text.len() + 1)
            .map(|tokens| tokens.len())
            .unwrap_or(usize::MAX)
    }

    /// 音声データを書き起こし
    /// samples: 16kHz, mono, f32の音声データ
    /// context: 認識精度向上のためのコンテキスト（プレイヤー名など）
    /// 戻り値: タイムスタンプ付きのテキストセグメント
    pub fn transcribe(
        &mut self,
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
        params.set_n_threads(thread_count() as i32);
        params.set_no_context(true);
        // 複数セグメント許可
        params.set_single_segment(false);
        // トークンごとの時刻を求めさせる。セグメントを句読点で区切り直すのに使う。
        params.set_token_timestamps(true);

        if let Some(ctx) = context {
            params.set_initial_prompt(ctx);
        }

        // 特殊トークン（タイムスタンプ等）の判定に使う。state を借りる前に取っておく。
        let first_special = self.ctx.token_eot();

        // 書き起こし実行
        if self.state.is_none() {
            self.state = Some(
                self.ctx
                    .create_state()
                    .map_err(TranscribeError::CreateState)?,
            );
        }
        let state = self.state.as_mut().expect("直前に用意している");

        state.full(params, samples).map_err(TranscribeError::Run)?;

        // トークンを時刻付きで集める
        let mut tokens = Vec::new();

        for segment in state.as_iter() {
            for index in 0..segment.n_tokens() {
                let Some(token) = segment.get_token(index) else {
                    continue;
                };

                let data = token.token_data();
                // タイムスタンプ等の特殊トークンは本文ではない
                if data.id >= first_special {
                    continue;
                }
                let Ok(text) = token.to_str_lossy() else {
                    continue;
                };

                // whisper.cpp のタイムスタンプはセンチ秒（10ms 単位）
                tokens.push(SpokenToken {
                    start_secs: data.t0 as f32 / 100.0,
                    end_secs: data.t1 as f32 / 100.0,
                    text: text.into_owned(),
                });
            }
        }

        Ok(split_into_segments(tokens))
    }
}

// =============================================================================
// ヘルパー関数
// =============================================================================

/// 書き起こしに使うスレッド数。
///
/// 書き起こしはゲーム本体・ボイスチャットと同時に動くため、割り当ての妥当性は
/// 環境によって変わる。実測しながら振れるよう `AMUS_WHISPER_THREADS` で上書きできる。
pub fn thread_count() -> usize {
    let available = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let requested = std::env::var("AMUS_WHISPER_THREADS")
        .ok()
        .and_then(|value| value.parse().ok());

    resolve_thread_count(available, requested)
}

/// 既定は論理コア数の半分。
///
/// 書き起こしの処理時間は音声長にほとんど依存しない固定費で、その大半は 30 秒窓の
/// エンコードが占める。スレッドを 8 から 12 へ増やしても処理時間は 1 割ほどしか
/// 縮まないため、残りのコアはゲーム側へ空ける。
fn resolve_thread_count(available: usize, requested: Option<usize>) -> usize {
    requested.unwrap_or((available / 2).clamp(4, 8)).max(1)
}

/// トークン列を句読点で区切ってセグメントにする。
///
/// Whisper が返すセグメントは無音の切れ目で決まるため、続けて喋ると数秒分が1つに
/// まとまり、どの発話がいつだったのかが失われる。メモは「いつ何を言ったか」を残す
/// ものなので、句読点で区切り直してトークンの時刻をそのまま持たせる。
fn split_into_segments(tokens: Vec<SpokenToken>) -> Vec<TranscriptionSegment> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut start_secs = 0.0;
    let mut end_secs = 0.0;

    for token in tokens {
        if text.is_empty() {
            start_secs = token.start_secs;
        }
        text.push_str(&token.text);
        end_secs = token.end_secs;

        if breaks_after(&token.text, &text) {
            push_segment(&mut segments, start_secs, end_secs, &text);
            text.clear();
        }
    }

    // 句読点で終わらなかった分
    if !text.is_empty() {
        push_segment(&mut segments, start_secs, end_secs, &text);
    }

    segments
}

/// 中身が残るなら整形してセグメントにする。
fn push_segment(segments: &mut Vec<TranscriptionSegment>, start: f32, end: f32, text: &str) {
    // 文字化け（置換文字）と効果音を落とす
    let clean: String = text.chars().filter(|c| *c != '\u{FFFD}').collect();
    let clean = remove_sound_effects(&clean);
    let clean = clean.trim_matches(|c: char| c.is_whitespace() || is_sentence_end(c));

    if clean.is_empty() {
        return;
    }

    segments.push(TranscriptionSegment {
        start_secs: start,
        end_secs: end,
        text: clean.to_string(),
    });
}

/// このトークンでセグメントを区切るか。`piece` はここまで溜めたテキスト。
///
/// 読点は息継ぎでも打たれるため、そこで必ず切ると「ラウンジで」「たぬころに遭遇」の
/// ように一続きの発話が分断される。助詞で終わっていれば文が続いているとみなして繋げる。
/// 句点は文の終わりなので助詞の後でも切る。
fn breaks_after(token_text: &str, piece: &str) -> bool {
    match token_text.chars().last() {
        Some(c) if is_full_stop(c) => true,
        Some(c) if is_comma(c) => !ends_with_particle(piece),
        _ => false,
    }
}

/// 助詞で終わっているか。区切り記号は落としてから見る。
///
/// 助詞で終わる名前（「かえで」など）は文の途中と誤判定するが、区切りが1つ減って
/// メモが長くなるだけで、分断されるより害が小さい。
fn ends_with_particle(piece: &str) -> bool {
    let trimmed = piece.trim_end_matches(is_sentence_end);

    matches!(
        trimmed.chars().last(),
        Some('で' | 'に' | 'と' | 'が' | 'は' | 'を' | 'へ' | 'の' | 'も')
    )
}

fn is_sentence_end(c: char) -> bool {
    is_comma(c) || is_full_stop(c)
}

fn is_comma(c: char) -> bool {
    matches!(c, '、' | '，' | ',')
}

fn is_full_stop(c: char) -> bool {
    matches!(c, '。' | '．' | '.' | '!' | '?' | '！' | '？')
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

#[cfg(test)]
mod tests {
    use super::*;

    /// (開始秒, 終了秒, テキスト) のトークン列を作る。
    fn tokens(items: &[(f32, f32, &str)]) -> Vec<SpokenToken> {
        items
            .iter()
            .map(|&(start_secs, end_secs, text)| SpokenToken {
                start_secs,
                end_secs,
                text: text.to_string(),
            })
            .collect()
    }

    fn summarize(segments: &[TranscriptionSegment]) -> Vec<(f32, &str)> {
        segments
            .iter()
            .map(|s| (s.start_secs, s.text.as_str()))
            .collect()
    }

    #[test]
    fn splits_at_punctuation_with_the_time_it_was_spoken() {
        let segments = split_into_segments(tokens(&[
            (0.0, 1.0, "エンジン"),
            (1.0, 1.8, "湧き"),
            (1.8, 2.0, "、"),
            (2.0, 2.6, "メイン"),
            (2.6, 2.8, "、"),
            (2.8, 4.2, "シャワーでしおりぬ"),
        ]));

        assert_eq!(
            summarize(&segments),
            [
                (0.0, "エンジン湧き"),
                (2.0, "メイン"),
                (2.8, "シャワーでしおりぬ"),
            ]
        );
    }

    #[test]
    fn the_last_piece_without_punctuation_still_becomes_a_segment() {
        let segments = split_into_segments(tokens(&[(0.0, 0.5, "はい"), (0.5, 1.2, "了解")]));

        assert_eq!(summarize(&segments), [(0.0, "はい了解")]);
    }

    #[test]
    fn a_segment_ends_when_its_last_token_ends() {
        let segments = split_into_segments(tokens(&[(0.0, 1.0, "メイン"), (1.0, 1.2, "。")]));

        assert_eq!(segments[0].end_secs, 1.2);
    }

    #[test]
    fn punctuation_only_pieces_are_dropped() {
        let segments = split_into_segments(tokens(&[
            (0.0, 0.2, "、"),
            (0.2, 1.0, "メイン"),
            (1.0, 1.2, "。"),
        ]));

        assert_eq!(summarize(&segments), [(0.2, "メイン")]);
    }

    #[test]
    fn a_piece_ending_in_a_particle_continues_into_the_next() {
        let segments = split_into_segments(tokens(&[
            (2.0, 2.8, "ラウンジ"),
            (2.8, 3.0, "で"),
            (3.0, 3.1, "、"),
            (3.1, 3.8, "たぬころ"),
            (3.8, 4.5, "に遭遇"),
        ]));

        // 「ラウンジで」で切ると一続きの発話が分断される
        assert_eq!(summarize(&segments), [(2.0, "ラウンジで、たぬころに遭遇")]);
    }

    #[test]
    fn a_full_stop_ends_a_piece_even_after_a_particle() {
        let segments = split_into_segments(tokens(&[
            (0.0, 0.8, "ラウンジ"),
            (0.8, 1.0, "で"),
            (1.0, 1.1, "。"),
            (1.1, 1.8, "メイン"),
        ]));

        assert_eq!(summarize(&segments), [(0.0, "ラウンジで"), (1.1, "メイン")]);
    }

    #[test]
    fn defaults_to_half_the_cores() {
        assert_eq!(resolve_thread_count(16, None), 8);
    }

    #[test]
    fn keeps_the_default_within_bounds() {
        // 少ないコアでも最低限の並列度は確保する
        assert_eq!(resolve_thread_count(2, None), 4);
        // 増やしても処理時間はほとんど縮まないため上限で止める
        assert_eq!(resolve_thread_count(32, None), 8);
    }

    #[test]
    fn an_explicit_request_wins_over_the_default() {
        assert_eq!(resolve_thread_count(8, Some(2)), 2);
        // 実測用なので上限を超える値も通す
        assert_eq!(resolve_thread_count(8, Some(16)), 16);
    }

    #[test]
    fn zero_threads_is_not_a_valid_request() {
        assert_eq!(resolve_thread_count(8, Some(0)), 1);
    }
}
