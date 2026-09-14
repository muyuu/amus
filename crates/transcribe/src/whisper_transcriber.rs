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

    /// 約1.4GB。GPU バックエンドを含むビルドで使う。
    ///
    /// distil-whisper-large-v3 ベースの日本語特化モデル（kotoba-whisper-v2.0）。
    /// エンコーダは large-v3 のフルサイズをそのまま使うため CPU では small より
    /// はるかに遅く実用にならないが、GPU ならその差はほぼ吸収される。ReazonSpeech
    /// （日本のTV音声）で学習されており、雑談寄りの音声で multilingual な
    /// large-v3/medium より高い精度が出るとされる。
    ///
    /// デコーダが2層しかなく、長い initial_prompt を渡すと生成が空になる
    /// （0セグメント）不具合が実測で確認されている。詳細は
    /// [`Self::prompt_token_budget`] を参照。
    pub const KOTOBA_V2: Self = Self {
        file_name: "ggml-kotoba-whisper-v2.0.bin",
        url: "https://huggingface.co/kotoba-tech/kotoba-whisper-v2.0-ggml/resolve/e3a0cf6a62b95911703cfb97d819292e058f12c3/ggml-kotoba-whisper-v2.0.bin",
        sha256: "eff70a8a236e731abba774ba71e1f6d0fce53302137208c32207e694e0bf4546",
        expected_bytes: 1_519_521_155,
    };

    /// モデルの通称（`small` / `kotoba-whisper-v2.0`）。ログや画面に出す。
    pub fn name(&self) -> &'static str {
        self.file_name
            .trim_start_matches("ggml-")
            .trim_end_matches(".bin")
    }

    /// `initial_prompt` に安全に渡せるトークン数の上限。
    ///
    /// アーキテクチャ上の上限は `n_text_ctx / 2`（[`crate::whisper_prompt::PROMPT_TOKEN_LIMIT`]、224）
    /// だが、これは「壊れずに載る」上限であって「壊れずに動く」上限ではない。
    /// kotoba-whisper-v2.0 は実機で 103トークンは正常、165トークンで生成が
    /// 空になる（0セグメント）ことを確認した。デコーダが2層しかなく、長い
    /// 条件文を正しく扱いきれないためと見られる。安全マージンを見て90に抑える。
    pub fn prompt_token_budget(&self) -> usize {
        if *self == Self::KOTOBA_V2 {
            90
        } else {
            crate::whisper_prompt::PROMPT_TOKEN_LIMIT
        }
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
        // whisper.cpp 内蔵の無音判定を無効化する。VAD（vad.rs）で発話と判定した区間
        // しか渡していないため二重にゲートする意味がなく、モデルによっては無音確率の
        // 較正がずれて全区間を無音扱いしてしまう（kotoba-whisper の 2 層デコーダで
        // 全セグメントが 0 件になる不具合が実際に起きた）。
        params.set_no_speech_thold(1.0);

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

/// 用意した音声サンプルで実際の書き起こし精度を確認する。
///
/// `test-audio/` 配下に `<名前>.mp3` と、期待する書き起こし文を書いた `<名前>.txt` を
/// 同名で置くと、それぞれ書き起こして期待テキストとの一致率を表示する。個人の録音を
/// 扱うためリポジトリには含めず（`.gitignore` 参照）、手元でのモデル比較用に使う。
/// CI では使わない。
///
/// 実行: cargo test --release --lib -- --ignored --nocapture verify_recognition
#[cfg(test)]
mod recognition_check {
    use super::*;
    use crate::resampler::Resampler;
    use crate::voice_recorder::SAMPLE_RATE;
    use crate::whisper_backend::TranscribeSetup;
    use std::path::Path;

    #[test]
    #[ignore]
    fn verify_recognition_against_fixtures() {
        let dir = Path::new("test-audio");
        if !dir.exists() {
            println!(
                "test-audio/ が見つからない。<名前>.mp3 と、期待する書き起こし文を書いた \
                 <名前>.txt を置くと使える。"
            );
            return;
        }

        let mut cases: Vec<_> = std::fs::read_dir(dir)
            .expect("test-audio の読み込みに失敗")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "mp3"))
            .collect();
        cases.sort();

        if cases.is_empty() {
            println!("test-audio/ に .mp3 が無い");
            return;
        }

        let mut transcriber =
            WhisperTranscriber::new(TranscribeSetup::CPU).expect("モデルの読み込みに失敗");

        let mut total_similarity = 0.0;
        for mp3_path in &cases {
            let txt_path = mp3_path.with_extension("txt");
            let expected = std::fs::read_to_string(&txt_path)
                .unwrap_or_else(|_| panic!("{} が無い", txt_path.display()));
            let expected = expected.trim();

            let samples = decode_mp3_as_16k_mono(mp3_path);
            let segments = transcriber
                .transcribe(&samples, None)
                .expect("書き起こしに失敗");
            let actual: String = segments.iter().map(|s| s.text.as_str()).collect();

            let sim = similarity(expected, &actual);
            total_similarity += sim;

            println!("\n[{}]", mp3_path.file_name().unwrap().to_string_lossy());
            println!("  期待: {expected}");
            println!("  実際: {actual}");
            println!("  一致率: {:.0}%", sim * 100.0);
        }

        println!(
            "\n=== 平均一致率: {:.0}% ({}件) ===",
            total_similarity / cases.len() as f64,
            cases.len()
        );
    }

    /// mp3 を読み、Whisper が要求する 16kHz mono f32 へ変換する。
    fn decode_mp3_as_16k_mono(path: &Path) -> Vec<f32> {
        use symphonia::core::audio::SampleBuffer;
        use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path)
            .unwrap_or_else(|e| panic!("{} を開けない: {e}", path.display()));
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("mp3");

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .expect("mp3 のフォーマット判定に失敗");
        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("音声トラックが無い")
            .clone();
        let track_id = track.id;
        let src_rate = track.codec_params.sample_rate.expect("サンプルレート不明");
        let channels = track
            .codec_params
            .channels
            .map(|c| c.count())
            .unwrap_or(1)
            .max(1);

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .expect("デコーダの作成に失敗");

        let mut mono = Vec::new();
        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            let Ok(decoded) = decoder.decode(&packet) else {
                continue;
            };

            let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            buf.copy_interleaved_ref(decoded);
            mono.extend(
                buf.samples()
                    .chunks(channels)
                    .map(|frame| frame.iter().sum::<f32>() / channels as f32),
            );
        }

        let mut resampler = Resampler::new(src_rate, SAMPLE_RATE);
        let mut out = Vec::new();
        resampler.process(&mono, |s| out.push(s));
        out
    }

    /// 文字ベースの編集距離から一致率(0.0〜1.0)を求める。句読点や表記揺れも
    /// 違いとして数えるため、大まかな目安として使う。
    fn similarity(expected: &str, actual: &str) -> f64 {
        let a: Vec<char> = expected.chars().collect();
        let b: Vec<char> = actual.chars().collect();
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            return 1.0;
        }

        1.0 - (levenshtein(&a, &b) as f64 / max_len as f64)
    }

    fn levenshtein(a: &[char], b: &[char]) -> usize {
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        let mut curr = vec![0; b.len() + 1];

        for (i, &ca) in a.iter().enumerate() {
            curr[0] = i + 1;
            for (j, &cb) in b.iter().enumerate() {
                let cost = if ca == cb { 0 } else { 1 };
                curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }

        prev[b.len()]
    }

    #[test]
    fn similarity_is_1_for_identical_text() {
        assert_eq!(similarity("エレキで会った", "エレキで会った"), 1.0);
    }

    #[test]
    fn similarity_drops_with_edits() {
        let sim = similarity("エレキで会った", "エレキで会あった");
        assert!(sim > 0.5 && sim < 1.0, "sim={sim}");
    }

    #[test]
    fn similarity_is_1_for_two_empty_strings() {
        assert_eq!(similarity("", ""), 1.0);
    }
}
