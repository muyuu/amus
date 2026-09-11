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

/// トリガーワード検出に応じて行うターン境界の操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TurnBoundary {
    /// ターンを開く
    Open,
    /// 進行中のターンを閉じる
    Close,
    /// 進行中のターンを閉じ、同じ位置で次のターンを開く
    Restart,
}

/// 検出をターン境界の操作へ変換する。
///
/// ターンは連続するため、進行中に開始ワードを検知したらそこで区切って次を開く。
/// 終了ワードは任意であり、開始ワードだけでも区間は途切れず確定する。
pub(super) struct BoundaryTracker {
    /// 直前の検出からこのサンプル数の間は次の検出を無視する。
    cooldown_samples: usize,
    last_detection: Option<usize>,
}

impl BoundaryTracker {
    pub(super) fn new(cooldown_samples: usize) -> Self {
        Self {
            cooldown_samples,
            last_detection: None,
        }
    }

    /// 検出を受け取り、行うべき境界操作を返す。`at` は検出位置の絶対インデックス。
    pub(super) fn accept(
        &mut self,
        hotword: Hotword,
        at: usize,
        turn_open: bool,
    ) -> Option<TurnBoundary> {
        if self.in_cooldown(at) {
            return None;
        }

        let boundary = match (hotword, turn_open) {
            (Hotword::TurnStart, false) => TurnBoundary::Open,
            (Hotword::TurnStart, true) => TurnBoundary::Restart,
            (Hotword::TurnEnd, true) => TurnBoundary::Close,
            // 開いていないターンは閉じられない
            (Hotword::TurnEnd, false) => return None,
        };

        self.last_detection = Some(at);
        Some(boundary)
    }

    /// 直前の検出から間もないか。同じ発話を重ねて拾うのを防ぐ。
    fn in_cooldown(&self, at: usize) -> bool {
        self.last_detection
            .is_some_and(|last| at.saturating_sub(last) < self.cooldown_samples)
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

    /// クールダウンは検証したい振る舞いではないので、十分短くしておく。
    fn tracker() -> BoundaryTracker {
        BoundaryTracker::new(0)
    }

    #[test]
    fn start_word_opens_a_turn() {
        assert_eq!(
            tracker().accept(Hotword::TurnStart, 100, false),
            Some(TurnBoundary::Open)
        );
    }

    #[test]
    fn start_word_during_a_turn_restarts_it() {
        assert_eq!(
            tracker().accept(Hotword::TurnStart, 100, true),
            Some(TurnBoundary::Restart)
        );
    }

    #[test]
    fn end_word_closes_an_open_turn() {
        assert_eq!(
            tracker().accept(Hotword::TurnEnd, 100, true),
            Some(TurnBoundary::Close)
        );
    }

    #[test]
    fn end_word_without_an_open_turn_is_ignored() {
        assert_eq!(tracker().accept(Hotword::TurnEnd, 100, false), None);
    }

    #[test]
    fn detections_during_cooldown_are_ignored() {
        let mut tracker = BoundaryTracker::new(50);

        assert_eq!(
            tracker.accept(Hotword::TurnStart, 100, false),
            Some(TurnBoundary::Open)
        );
        // 同じ発話を重ねて拾わない
        assert_eq!(tracker.accept(Hotword::TurnStart, 140, true), None);
        // クールダウンを過ぎれば拾う
        assert_eq!(
            tracker.accept(Hotword::TurnStart, 150, true),
            Some(TurnBoundary::Restart)
        );
    }

    #[test]
    fn ignored_detections_do_not_extend_the_cooldown() {
        let mut tracker = BoundaryTracker::new(50);
        tracker.accept(Hotword::TurnStart, 100, false);

        // 無視された検出でクールダウンの起点がずれると、次が拾えなくなる
        tracker.accept(Hotword::TurnStart, 140, true);

        assert_eq!(
            tracker.accept(Hotword::TurnStart, 150, true),
            Some(TurnBoundary::Restart)
        );
    }
}
