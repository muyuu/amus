//! トリガーワードの照合
//!
//! 書き起こしたテキストにトリガーワードが含まれるかを判定する。音声認識の結果は
//! 表記が揺れるため、読みへ正規化した上で、多少の誤りを許して照合する。

use super::reading::{match_range, read};

/// 検出したトリガーワードの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hotword {
    TurnStart,
    TurnEnd,
}

/// トリガーワードの照合器。
///
/// 1つのトリガーワードに複数の表記を登録できる。漢字の読みは求められないため
/// （「ターン開始」と「ターンかいし」は別物として扱われる）、想定する表記を並べて登録する。
pub(super) struct HotwordMatcher {
    start_forms: Vec<Vec<char>>,
    end_forms: Vec<Vec<char>>,
}

impl HotwordMatcher {
    pub(super) fn new(start_words: &[&str], end_words: &[&str]) -> Self {
        let reading = |w: &&str| read(w).chars;

        Self {
            start_forms: start_words.iter().map(reading).collect(),
            end_forms: end_words.iter().map(reading).collect(),
        }
    }

    /// テキストにトリガーワードが含まれていればその位置とともに返す。
    pub(super) fn find(&self, text: &str) -> Option<HotwordMatch> {
        let reading = read(text);

        // 開始と終了が同時に含まれることは想定しない。先に開始を見る。
        for (forms, hotword) in [
            (&self.start_forms, Hotword::TurnStart),
            (&self.end_forms, Hotword::TurnEnd),
        ] {
            let Some((start, end)) = forms
                .iter()
                .filter_map(|form| match_range(&reading.chars, form))
                .min_by_key(|&(_, end)| end)
            else {
                continue;
            };

            return Some(HotwordMatch {
                hotword,
                word_start: reading.source_end(start),
                word_end: reading.source_end(end),
            });
        }
        None
    }
}

/// 見つかったトリガーワード。位置は元テキストの文字位置。
///
/// 1つの認識結果には語の前後の発話も入りうる。範囲で返すことで、前はこれから閉じる
/// ターンへ、後ろは開いたターンへ、それぞれ残せる。
pub(super) struct HotwordMatch {
    pub(super) hotword: Hotword,
    /// 語の先頭
    pub(super) word_start: usize,
    /// 語の直後
    pub(super) word_end: usize,
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

/// 既定のトリガーワード。
///
/// 漢字とかなの違いは照合で吸収できないため、想定する表記を並べる。
pub(super) const DEFAULT_START_WORDS: &[&str] = &["ターン開始", "ターンかいし"];
pub(super) const DEFAULT_END_WORDS: &[&str] = &["ターン終了", "ターンしゅうりょう"];

/// 認識プロンプトに載せる代表表記。
///
/// 照合用の表記ゆれは載せない。プロンプトはモデルに出させたい表記を示すものであり、
/// 揺れを並べると的が分散する。
pub(super) const PROMPT_WORDS: &[&str] = &[DEFAULT_START_WORDS[0], DEFAULT_END_WORDS[0]];

/// 同じ発話を重ねて拾わないためのクールダウン。
pub(super) const COOLDOWN_SECS: f32 = 3.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher() -> HotwordMatcher {
        HotwordMatcher::new(&["ターン開始"], &["ターン終了"])
    }

    #[test]
    fn absorbs_a_long_vowel_written_as_a_kana() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // 「ター」と「たあ」は同じ読み
        assert_eq!(
            matcher.find("たあんかいし").map(|m| m.hotword),
            Some(Hotword::TurnStart)
        );
    }

    #[test]
    fn tolerates_a_single_misheard_character() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // 実際の認識結果。「ターン」が「はあん」に化けている
        assert_eq!(
            matcher.find("はあん、かいし").map(|m| m.hotword),
            Some(Hotword::TurnStart)
        );
    }

    #[test]
    fn does_not_match_unrelated_speech_of_a_similar_length() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        assert!(matcher.find("さっきのタスクやった").is_none());
        assert!(matcher.find("カフェテリアにいた").is_none());
    }

    #[test]
    fn finds_the_start_word_in_surrounding_text() {
        assert_eq!(
            matcher().find("じゃあターン開始します").map(|m| m.hotword),
            Some(Hotword::TurnStart)
        );
    }

    #[test]
    fn absorbs_kana_and_separator_differences() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // ひらがな・カタカナの違いを無視する
        assert_eq!(
            matcher.find("ターンカイシ").map(|m| m.hotword),
            Some(Hotword::TurnStart)
        );
        // 区切り記号や空白が混ざっても拾う
        assert_eq!(
            matcher.find("ターン、しゅう りょう").map(|m| m.hotword),
            Some(Hotword::TurnEnd)
        );
    }

    #[test]
    fn returns_none_without_a_trigger_word() {
        assert!(matcher().find("エレキで死体見つけた").is_none());
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
