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
}
