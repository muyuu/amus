//! 書き起こし結果の語彙補正
//!
//! 認識プロンプトに語を載せても、その表記で出てくるとは限らない。読みが合っていれば
//! 既知語彙の正規表記へ寄せる。プロンプトのトークン予算とは無関係に効くため、予算に
//! 収まらず載せられなかった語も拾える。

use super::reading::read;

/// 既知語彙で書き起こしを補正する。
///
/// 語彙はプレイヤー名・部屋名・ゲーム用語。ゲームごとに変わるため作り直して使う。
pub(super) struct VocabularyCorrector {
    /// 正規表記と、その読み。読みの長い順に並べる。
    terms: Vec<(String, Vec<char>)>,
}

impl VocabularyCorrector {
    pub(super) fn new<'a>(terms: impl IntoIterator<Item = &'a str>) -> Self {
        let mut terms: Vec<(String, Vec<char>)> = terms
            .into_iter()
            .map(|term| (term.to_string(), read(term).chars))
            .filter(|(_, reading)| !reading.is_empty())
            .collect();

        // 長い語から当てる。短い語が長い語の一部に先に当たるのを防ぐ。
        terms.sort_by_key(|(_, reading)| std::cmp::Reverse(reading.len()));

        Self { terms }
    }

    /// 読みの一致する箇所を正規表記へ置き換えた文字列を返す。
    ///
    /// 置き換えは読みが完全に一致する箇所に限る。誤って書き換えると発話そのものが
    /// 失われるため、取りこぼしより誤爆を避ける。
    pub(super) fn correct(&self, text: &str) -> String {
        let reading = read(text);
        let chars: Vec<char> = text.chars().collect();

        // 置換済みの読み位置。長い語から当てるため、内側に短い語を重ねて当てない。
        let mut taken = vec![false; reading.chars.len()];
        // 元テキストの文字位置ごとの置換内容。開始位置に正規表記、残りは削除。
        let mut replacement: Vec<Option<&str>> = vec![None; chars.len()];
        let mut removed = vec![false; chars.len()];

        for (canonical, term) in &self.terms {
            for start in 0..reading.chars.len().saturating_sub(term.len() - 1) {
                let end = start + term.len();
                if reading.chars[start..end] != term[..] {
                    continue;
                }
                if taken[start..end].iter().any(|&t| t) {
                    continue;
                }
                taken[start..end].iter_mut().for_each(|t| *t = true);

                // 読みの範囲を元テキストの範囲へ戻す。起点は語の先頭そのものを指す
                // 必要がある。直前の文字の直後から取ると、間に落とした区切り記号が
                // あったときにそれごと消してしまう。
                let from = reading.source_start(start);
                let to = reading.source_end(end);
                replacement[from] = Some(canonical);
                removed[from + 1..to].iter_mut().for_each(|r| *r = true);
            }
        }

        let mut corrected = String::with_capacity(text.len());
        for (index, c) in chars.iter().enumerate() {
            match replacement[index] {
                Some(canonical) => corrected.push_str(canonical),
                None if removed[index] => {}
                None => corrected.push(*c),
            }
        }
        corrected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corrector() -> VocabularyCorrector {
        VocabularyCorrector::new(["エレキ", "エンジン", "カフェテリア"])
    }

    #[test]
    fn writes_a_kana_heard_word_in_its_known_form() {
        // プロンプトに載せても、その表記で出てくるとは限らない
        assert_eq!(corrector().correct("えれきにいた"), "エレキにいた");
    }

    #[test]
    fn corrects_a_word_that_never_fit_in_the_prompt() {
        // 部屋名はトークン予算を超えて落ちるが、補正は予算と無関係に効く
        assert_eq!(corrector().correct("えんじんで死体"), "エンジンで死体");
    }

    #[test]
    fn leaves_a_word_that_is_already_written_correctly() {
        assert_eq!(corrector().correct("エレキにいた"), "エレキにいた");
    }

    #[test]
    fn prefers_the_longer_word_when_two_overlap() {
        // 「エレキ」が先に当たると「ガチャガチャ」が取り残される
        let corrector = VocabularyCorrector::new(["エレキ", "エレキガチャガチャ"]);

        assert_eq!(
            corrector.correct("えれきがちゃがちゃで会った"),
            "エレキガチャガチャで会った"
        );
    }

    #[test]
    fn absorbs_a_long_vowel_written_as_a_kana() {
        let corrector = VocabularyCorrector::new(["カフェテリア", "ターン"]);

        assert_eq!(corrector.correct("たあんの話"), "ターンの話");
    }

    #[test]
    fn keeps_punctuation_that_sits_before_the_word() {
        // 読みは区切り記号を落とすため、元の位置へ戻すときに巻き込みやすい
        assert_eq!(
            corrector().correct("そこで、えれきに行った"),
            "そこで、エレキに行った"
        );
    }

    #[test]
    fn leaves_speech_that_matches_nothing() {
        assert_eq!(
            corrector().correct("さっきタスクやってた"),
            "さっきタスクやってた"
        );
    }

    /// 候補モデルの生の書き起こし結果に語彙補正をかけ、実際の語彙（プレイヤー名・
    /// 部屋名・用語）がどれだけ復元できるかを比較する。
    /// 実行: cargo test --lib -- --ignored --nocapture compare_candidate_models
    #[test]
    #[ignore]
    fn compare_candidate_models() {
        use crate::i18n::words::ja::JapaneseWords;

        let player_names = [
            "りょーちゃん",
            "えんがわ",
            "しおりぬ",
            "なあこ",
            "ふく",
            "たぬころ",
            "にゃんばる",
            "れもん",
            "かえで",
            "ぼっくり",
            "かいくん",
            "にこ",
        ];
        let rooms = JapaneseWords::AIRSHIP_ROOMS
            .iter()
            .copied()
            .filter(|s| !s.chars().all(|c| c.is_ascii_alphabetic() || c == ' '));
        let terms = [
            JapaneseWords::SABOTAGE,
            JapaneseWords::SABOTAGE_COMMS,
            JapaneseWords::SABOTAGE_LIGHTS,
            JapaneseWords::SABOTAGE_O2,
            JapaneseWords::SABOTAGE_REACTOR,
            JapaneseWords::SABOTAGE_DOORS,
            JapaneseWords::SPAWN,
            JapaneseWords::VENT,
            JapaneseWords::TASK,
            JapaneseWords::DISCUSSION,
        ];

        let known: Vec<&str> = player_names
            .iter()
            .copied()
            .chain(terms.iter().copied())
            .chain(rooms)
            .collect();
        let corrector = VocabularyCorrector::new(known.iter().copied());

        // sample01.wav を各モデルでそのまま書き起こした生の結果（プロンプト無し）。
        let raw_transcripts: [(&str, &str); 5] = [
            (
                "small",
                "ターン開始行エンジンメインリョーちゃんに相談カモツエレキでカイ君に相談セキューキッチンでナーコに相談ブキコでしおりに相談ターン終了ご視ありがとうございました",
            ),
            (
                "medium(現行GPU)",
                "ターン開始 金エンジン インでちゃんに物 エレキでカイ君にセキュ キッチンでナアコに武器でしおりに ターン終了ご視ありがとうございました",
            ),
            (
                "kotoba-whisper-v2.0",
                "ターン開始キンコ脇エンジンメインでりょうに遭遇ターン終了ターン終了エレキで開くんに遭遇",
            ),
            (
                "whisper-large-v3-turbo-ja",
                "ターン開始金エンジンメインでちゃんに物エレキで海君に石油キッチンでナアコに武器でしおりにターン終了ありがとうございました",
            ),
            (
                "anime-whisper",
                "unboxing回し金切エンジンメインで良ちゃんに物…エレキで開くんに石油キッチンで子にドキ子でしおりにターン終了…失料S点香は照れていた",
            ),
        ];

        // test-audio/live_chunk_000〜009.wav（実ライブ音声、短い断片10個）を
        // それぞれ書き起こしたものを連結（プロンプト無し）。
        let chunk_transcripts: [(&str, &str); 2] = [
            (
                "small(live chunks)",
                "ターン開始 金 んじん ですよりにそう キッチ エレキでレモニソーグ 次の週にお会いしましょう はぁ やりにくい",
            ),
            (
                "medium(live chunks)",
                "ターン開始 金 うんじん で理にそこに行ってください では以上です きっちー エレキでレモンに業 ターン終了 んー… はい よっこい おれ様でした",
            ),
        ];

        for (label, raw) in raw_transcripts.into_iter().chain(chunk_transcripts) {
            let corrected = corrector.correct(raw);
            let hits: Vec<&str> = known
                .iter()
                .copied()
                .filter(|term| corrected.contains(term))
                .collect();
            let hotwords: Vec<&str> = ["ターン開始", "ターン終了"]
                .into_iter()
                .filter(|w| raw.contains(w))
                .collect();
            println!(
                "\n[{label}] {}/{} 復元: {:?} / ホットワード: {:?}",
                hits.len(),
                known.len(),
                hits,
                hotwords
            );
            println!("  補正後: {corrected}");
        }
    }

    /// `sample01.wav` をアプリと同じVAD（[`transcribe::speech::SpeechDetector`]）で短い発話単位に
    /// 切り出してから、各モデルで独立に（`no_context=true`で、状態は使い回して）書き起こし、
    /// 語彙補正後の復元数とホットワード検知を比較する。全体を1回で渡す（whisper.cpp内蔵の
    /// 長尺アルゴリズムに乗る）テストとは条件が異なるため、実運用に近いのはこちら。
    /// 実行: cargo test --release --lib -- --ignored --nocapture compare_with_real_chunking
    #[test]
    #[ignore]
    fn compare_with_real_chunking() {
        use crate::i18n::words::ja::JapaneseWords;
        use transcribe::speech::SpeechDetector;
        use transcribe::whisper_backend::TranscribeSetup;
        use transcribe::whisper_transcriber::WhisperModel;
        use transcribe::WhisperTranscriber;

        // sample01.wav を読み、16kHz mono へ変換する。
        let mut reader = hound::WavReader::open("sample01.wav").expect("wavを開けない");
        let spec = reader.spec();
        let max_val = (1i64 << (spec.bits_per_sample.max(1) - 1)) as f32;
        let raw: Vec<f32> = reader
            .samples::<i32>()
            .map(|s| s.expect("読み取り失敗") as f32 / max_val)
            .collect();
        let mut resampler = transcribe::resampler::Resampler::new(spec.sample_rate, 16000);
        let mut samples = Vec::new();
        resampler.process(&raw, |s| samples.push(s));

        // vad.rs と同じ窓幅でRMSを測り、SpeechDetectorに通す。
        const WINDOW_SECS: f32 = 0.1;
        let window_len = (16000.0 * WINDOW_SECS) as usize;
        let mut detector = SpeechDetector::default();
        let mut chunks: Vec<(usize, usize)> = Vec::new();
        let mut pos = 0;
        while pos + window_len <= samples.len() {
            let window = pos..pos + window_len;
            let rms = {
                let s = &samples[window.clone()];
                (s.iter().map(|&x| x * x).sum::<f32>() / s.len() as f32).sqrt()
            };
            if let Some(speech) = detector.observe(window.clone(), rms) {
                chunks.push((speech.start, speech.end.min(samples.len())));
            }
            pos = window.end;
        }
        println!("検出された発話区間: {}個", chunks.len());

        let player_names = [
            "りょーちゃん",
            "えんがわ",
            "しおりぬ",
            "なあこ",
            "ふく",
            "たぬころ",
            "にゃんばる",
            "れもん",
            "かえで",
            "ぼっくり",
            "かいくん",
            "にこ",
        ];
        let rooms = JapaneseWords::AIRSHIP_ROOMS
            .iter()
            .copied()
            .filter(|s| !s.chars().all(|c| c.is_ascii_alphabetic() || c == ' '));
        let known: Vec<&str> = player_names.iter().copied().chain(rooms).collect();
        let corrector = VocabularyCorrector::new(known.iter().copied());

        for (label, setup) in [
            ("small", TranscribeSetup::CPU),
            ("medium", {
                TranscribeSetup {
                    gpu: None,
                    model: WhisperModel::MEDIUM,
                }
            }),
        ] {
            let mut transcriber = WhisperTranscriber::new(setup).expect("モデルの読み込みに失敗");
            let mut all_text = String::new();
            for &(start, end) in &chunks {
                let segments = transcriber
                    .transcribe(&samples[start..end], None)
                    .expect("書き起こしに失敗");
                for seg in segments {
                    all_text.push_str(&seg.text);
                    all_text.push(' ');
                }
            }

            let corrected = corrector.correct(&all_text);
            let hits: Vec<&str> = known
                .iter()
                .copied()
                .filter(|term| corrected.contains(term))
                .collect();
            let hotwords: Vec<&str> = ["ターン開始", "ターン終了"]
                .into_iter()
                .filter(|w| all_text.contains(w))
                .collect();
            println!(
                "\n[{label}] {}/{} 復元: {:?} / ホットワード: {:?}",
                hits.len(),
                known.len(),
                hits,
                hotwords
            );
            println!("  生: {all_text}");
            println!("  補正後: {corrected}");
        }
    }
}
