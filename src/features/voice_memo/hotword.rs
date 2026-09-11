//! トリガーワードの照合
//!
//! 書き起こしたテキストにトリガーワードが含まれるかを判定する。音声認識の結果は
//! 表記が揺れるため、ひらがな・カタカナの違いや区切り記号は無視して照合する。

/// 検出したトリガーワードの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hotword {
    TurnStart,
    TurnEnd,
}

/// トリガーワードの照合器。
///
/// 1つのトリガーワードに複数の表記を登録できる。漢字とかなの違いは吸収できないため
/// （「ターン開始」と「ターンかいし」は別物として扱われる）、想定する表記を並べて登録する。
pub(super) struct HotwordMatcher {
    start_forms: Vec<String>,
    end_forms: Vec<String>,
}

impl HotwordMatcher {
    pub(super) fn new(start_words: &[&str], end_words: &[&str]) -> Self {
        Self {
            start_forms: start_words.iter().map(|w| normalize(w)).collect(),
            end_forms: end_words.iter().map(|w| normalize(w)).collect(),
        }
    }

    /// テキストにトリガーワードが含まれていればその種類を返す。
    pub(super) fn find(&self, text: &str) -> Option<Hotword> {
        let text = normalize(text);

        // 開始と終了が同時に含まれることは想定しない。先に開始を見る。
        if self.start_forms.iter().any(|form| text.contains(form)) {
            return Some(Hotword::TurnStart);
        }
        if self.end_forms.iter().any(|form| text.contains(form)) {
            return Some(Hotword::TurnEnd);
        }
        None
    }
}

/// 照合用に表記を揃える。
///
/// 音声認識の結果は区切り記号や空白が入りうるため落とし、ひらがなはカタカナへ寄せる。
fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !is_separator(*c))
        .map(to_katakana)
        .collect()
}

fn is_separator(c: char) -> bool {
    matches!(
        c,
        '、' | '。' | '，' | '．' | ',' | '.' | '!' | '?' | '！' | '？' | '・' | '-'
    )
}

/// ひらがなをカタカナへ寄せる。カタカナ・漢字・その他はそのまま。
fn to_katakana(c: char) -> char {
    match c {
        'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher() -> HotwordMatcher {
        HotwordMatcher::new(&["ターン開始"], &["ターン終了"])
    }

    #[test]
    fn finds_the_start_word_in_surrounding_text() {
        assert_eq!(
            matcher().find("じゃあターン開始します"),
            Some(Hotword::TurnStart)
        );
    }

    #[test]
    fn absorbs_kana_and_separator_differences() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // ひらがな・カタカナの違いを無視する
        assert_eq!(matcher.find("ターンカイシ"), Some(Hotword::TurnStart));
        // 区切り記号や空白が混ざっても拾う
        assert_eq!(
            matcher.find("ターン、しゅう りょう"),
            Some(Hotword::TurnEnd)
        );
    }

    #[test]
    fn returns_none_without_a_trigger_word() {
        assert_eq!(matcher().find("エレキで死体見つけた"), None);
    }
}
